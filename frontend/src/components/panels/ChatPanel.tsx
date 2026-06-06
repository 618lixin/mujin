import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  clearChatHistory,
  getChatDays,
  getChatHistory,
  startChatStream,
} from "../../features/api/chat";
import { getPendingMessage } from "../../features/api/memory";
import type {
  ChatDaySummary,
  ChatMessage,
  EmotionResult,
  PendingMessage,
} from "../../features/api/types";
import { DEFAULT_USER_ID } from "../../features/api/types";

type LocalMessage = ChatMessage & {
  id: string;
  meta?: string;
  pending?: boolean;
  proactive?: boolean;
  eventCreated?: boolean;
};

interface TokenPayload {
  streamId: string;
  token: string;
}

interface DonePayload {
  streamId: string;
  meta: {
    emotion: EmotionResult;
    turnCount: number;
  };
}

interface ErrorPayload {
  streamId: string;
  error: string;
}

function messageId(prefix: string) {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

function todayString() {
  const now = new Date();
  const year = now.getFullYear();
  const month = String(now.getMonth() + 1).padStart(2, "0");
  const day = String(now.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function formatDayLabel(date: string) {
  return date === todayString() ? "今天" : date;
}

function eventWasCreated(emotion?: EmotionResult): boolean {
  if (!emotion) return false;
  return emotion.importance >= 0.6 && !!emotion.eventType;
}

function toLocalMessages(history: ChatMessage[]): LocalMessage[] {
  return history
    .filter((msg) => msg.role === "user" || msg.role === "assistant")
    .map((msg, index) => ({ ...msg, id: `history-${index}` }));
}

function mergeTodayIntoDays(days: ChatDaySummary[]) {
  const today = todayString();
  return days.some((day) => day.date === today) ? days : [{ date: today, messageCount: 0 }, ...days];
}

export function ChatPanel() {
  const [messages, setMessages] = useState<LocalMessage[]>([]);
  const [chatDays, setChatDays] = useState<ChatDaySummary[]>([]);
  const [selectedDate, setSelectedDate] = useState(todayString());
  const [input, setInput] = useState("");
  const [isStreaming, setIsStreaming] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const scrollerRef = useRef<HTMLDivElement>(null);
  const activeStreamIdRef = useRef<string | null>(null);
  const activeAssistantIdRef = useRef<string | null>(null);
  const selectedDateRef = useRef(selectedDate);
  const historyLoadSeqRef = useRef(0);
  const today = todayString();
  const isViewingToday = selectedDate === today;

  function claimStreamEvent(streamId: string) {
    if (!activeAssistantIdRef.current) return null;
    if (!activeStreamIdRef.current) {
      activeStreamIdRef.current = streamId;
    }
    return activeStreamIdRef.current === streamId ? activeAssistantIdRef.current : null;
  }

  async function refreshChatDays() {
    const days = await getChatDays();
    setChatDays(mergeTodayIntoDays(days));
  }

  async function loadHistoryForDate(date: string, loadSeq: number) {
    const history = await getChatHistory(DEFAULT_USER_ID, date);
    if (loadSeq !== historyLoadSeqRef.current) return;
    setMessages(toLocalMessages(history));
  }

  function appendPendingMessage(pending: PendingMessage) {
    setMessages((current) => [
      ...current,
      {
        id: messageId("proactive"),
        role: "assistant",
        content: pending.message,
        meta: pending.reason,
        proactive: true,
      },
    ]);
  }

  useEffect(() => {
    let disposed = false;
    const initialDate = todayString();

    void Promise.all([getChatDays(), getChatHistory(DEFAULT_USER_ID, initialDate)])
      .then(([days, history]) => {
        if (disposed) return;
        setChatDays(mergeTodayIntoDays(days));
        setSelectedDate(initialDate);
        setMessages(toLocalMessages(history));
      })
      .catch((err: unknown) => setError(err instanceof Error ? err.message : String(err)));

    void getPendingMessage().then((pending) => {
      if (!disposed && pending) appendPendingMessage(pending);
    });

    return () => {
      disposed = true;
    };
  }, []);

  useEffect(() => {
    selectedDateRef.current = selectedDate;
  }, [selectedDate]);

  useEffect(() => {
    const unlisteners: Array<() => void> = [];
    let disposed = false;

    void Promise.all([
      listen<TokenPayload>("chat-token", (event) => {
        const assistantId = claimStreamEvent(event.payload.streamId);
        if (!assistantId) return;
        setMessages((current) =>
          current.map((msg) =>
            msg.id === assistantId
              ? { ...msg, content: msg.content + event.payload.token, pending: false }
              : msg,
          ),
        );
      }),
      listen<DonePayload>("chat-done", (event) => {
        const assistantId = claimStreamEvent(event.payload.streamId);
        if (!assistantId) return;
        const created = eventWasCreated(event.payload.meta.emotion);
        setMessages((current) =>
          current.map((msg) =>
            msg.id === assistantId ? { ...msg, pending: false, eventCreated: created } : msg,
          ),
        );
        activeStreamIdRef.current = null;
        activeAssistantIdRef.current = null;
        setIsStreaming(false);
        void refreshChatDays();
      }),
      listen<ErrorPayload>("chat-error", (event) => {
        const assistantId = claimStreamEvent(event.payload.streamId);
        if (!assistantId) return;
        setMessages((current) =>
          current.map((msg) =>
            msg.id === assistantId
              ? { ...msg, pending: false, content: msg.content || `发送失败：${event.payload.error}` }
              : msg,
          ),
        );
        setError(event.payload.error);
        activeStreamIdRef.current = null;
        activeAssistantIdRef.current = null;
        setIsStreaming(false);
      }),
      listen<{ userId: string; message: string }>("proactive-message", (event) => {
        if (event.payload.userId !== DEFAULT_USER_ID) return;
        if (selectedDateRef.current !== todayString()) return;
        appendPendingMessage({
          message: event.payload.message,
          reason: "proactive",
          createdAt: new Date().toISOString(),
        });
      }),
    ]).then((items) => {
      if (disposed) {
        items.forEach((unlisten) => unlisten());
      } else {
        unlisteners.push(...items);
      }
    });

    return () => {
      disposed = true;
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, []);

  useEffect(() => {
    scrollerRef.current?.scrollTo({ top: scrollerRef.current.scrollHeight });
  }, [messages]);

  async function handleSend() {
    const text = input.trim();
    if (!text || isStreaming || !isViewingToday) return;

    setError(null);
    setSelectedDate(todayString());
    setIsStreaming(true);
    setInput("");
    const assistantId = messageId("assistant");
    setMessages((current) => [
      ...current,
      { id: messageId("user"), role: "user", content: text },
      { id: assistantId, role: "assistant", content: "", pending: true },
    ]);
    activeAssistantIdRef.current = assistantId;
    activeStreamIdRef.current = null;

    try {
      const streamId = await startChatStream(text);
      if (activeAssistantIdRef.current === assistantId) {
        activeStreamIdRef.current = streamId;
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      setMessages((current) =>
        current.map((msg) =>
          msg.id === assistantId ? { ...msg, content: `发送失败：${message}`, pending: false } : msg,
        ),
      );
      activeAssistantIdRef.current = null;
      activeStreamIdRef.current = null;
      setIsStreaming(false);
    }
  }

  async function handleClear() {
    await clearChatHistory(DEFAULT_USER_ID, selectedDate);
    activeAssistantIdRef.current = null;
    activeStreamIdRef.current = null;
    setIsStreaming(false);
    setMessages([]);
    await refreshChatDays();
  }

  async function handleSelectDate(date: string) {
    if (isStreaming) return;
    const loadSeq = historyLoadSeqRef.current + 1;
    historyLoadSeqRef.current = loadSeq;
    setError(null);
    setSelectedDate(date);
    activeAssistantIdRef.current = null;
    activeStreamIdRef.current = null;
    await loadHistoryForDate(date, loadSeq);
  }

  return (
    <div className="flex-1 min-h-0 flex bg-paper/30">
      <aside className="w-44 shrink-0 border-r border-paper-deep/20 bg-paper/40 flex flex-col">
        <div className="h-10 px-4 border-b border-paper-deep/20 flex items-center">
          <h3 className="text-[12px] font-display font-medium text-ink-soft">历史记录</h3>
        </div>
        <div className="flex-1 min-h-0 overflow-y-auto p-2 space-y-1">
          {chatDays.map((day) => (
            <button
              key={day.date}
              type="button"
              onClick={() => void handleSelectDate(day.date)}
              disabled={isStreaming}
              className={`w-full rounded-lg px-3 py-2 text-left transition-colors ${
                day.date === selectedDate
                  ? "bg-bamboo text-cloud"
                  : "text-ink-soft hover:bg-cloud/70"
              }`}
            >
              <div className="text-[12px] font-medium">{formatDayLabel(day.date)}</div>
              <div
                className={`text-[10px] ${
                  day.date === selectedDate ? "text-cloud/75" : "text-ink-ghost"
                }`}
              >
                {day.messageCount} 条消息
              </div>
            </button>
          ))}
        </div>
      </aside>

      <div className="flex-1 min-w-0 min-h-0 flex flex-col">
        <div className="h-10 px-5 border-b border-paper-deep/20 flex items-center justify-between shrink-0">
          <div>
            <h2 className="text-[13px] font-display font-medium text-ink-soft">
              对话 · {formatDayLabel(selectedDate)}
            </h2>
            {error && <p className="text-[10px] text-red-400">{error}</p>}
            {!isViewingToday && (
              <p className="text-[10px] text-ink-ghost">历史对话只读，回到今天后继续聊天。</p>
            )}
          </div>
          <button
            type="button"
            onClick={() => void handleClear()}
            className="h-7 px-3 rounded-lg text-[11px] text-ink-ghost hover:text-red-500 hover:bg-danger-bg transition-colors"
          >
            清空
          </button>
        </div>

        <div ref={scrollerRef} className="flex-1 min-h-0 overflow-y-auto px-6 py-5 space-y-3">
          {messages.length === 0 ? (
            <div className="h-full flex items-center justify-center text-[13px] text-ink-ghost">
              {isViewingToday ? "今天发生了什么，直接说就好。" : "这一天还没有对话记录。"}
            </div>
          ) : (
            messages.map((msg) => (
              <div
                key={msg.id}
                className={`flex ${msg.role === "user" ? "justify-end" : "justify-start"}`}
              >
                <div
                  className={`max-w-[72%] rounded-xl px-3.5 py-2 text-[13px] leading-7 whitespace-pre-wrap break-words border ${
                    msg.role === "user"
                      ? "bg-bamboo text-cloud border-bamboo"
                      : msg.proactive
                        ? "bg-bamboo-mist/70 text-ink-soft border-bamboo/20"
                        : "bg-cloud/85 text-ink-soft border-paper-deep/40"
                  }`}
                >
                  {msg.pending && !msg.content ? "正在想..." : msg.content}
                  {msg.eventCreated && (
                    <div className="mt-2 pt-2 border-t border-bamboo/30 text-[11px] text-bamboo flex items-center gap-1.5">
                      <svg
                        width="12"
                        height="12"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        strokeWidth="2.5"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                      >
                        <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
                        <polyline points="22 4 12 14.01 9 11.01" />
                      </svg>
                      <span className="font-medium">已保存到记忆</span>
                    </div>
                  )}
                </div>
              </div>
            ))
          )}
        </div>

        <div className="shrink-0 border-t border-paper-deep/25 px-5 py-3 bg-paper/50">
          <div className="flex gap-2 rounded-xl border border-paper-deep/40 bg-cloud/70 px-3 py-2">
            <textarea
              value={input}
              onChange={(event) => setInput(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter" && !event.shiftKey) {
                  event.preventDefault();
                  void handleSend();
                }
              }}
              rows={2}
              placeholder={
                isViewingToday ? "输入一句话，Enter 发送，Shift+Enter 换行" : "历史对话只读，请回到今天继续聊天"
              }
              className="flex-1 min-h-[44px] max-h-[120px] text-[13px] leading-6 text-ink-soft placeholder:text-ink-ghost/60"
              disabled={isStreaming || !isViewingToday}
            />
            <button
              type="button"
              onClick={() => void handleSend()}
              disabled={!input.trim() || isStreaming || !isViewingToday}
              className="self-end h-8 px-4 rounded-lg bg-bamboo text-cloud text-[12px] disabled:opacity-40 disabled:cursor-not-allowed hover:bg-bamboo-light transition-colors"
            >
              发送
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
