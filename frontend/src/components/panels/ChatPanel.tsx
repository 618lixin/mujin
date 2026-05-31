import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { clearChatHistory, getChatHistory, startChatStream } from "../../features/api/chat";
import { getPendingMessage } from "../../features/api/memory";
import type { ChatMessage, EmotionResult, PendingMessage } from "../../features/api/types";
import { DEFAULT_USER_ID } from "../../features/api/types";

type LocalMessage = ChatMessage & {
  id: string;
  meta?: string;
  pending?: boolean;
  proactive?: boolean;
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

function emotionMeta(emotion?: EmotionResult): string {
  if (!emotion) return "";
  const parts: string[] = [];
  if (emotion.emotions.length > 0) parts.push(`情绪 ${emotion.emotions.join(" / ")}`);
  if (emotion.importance >= 0.6) parts.push("已沉淀为事件");
  return parts.join(" · ");
}

export function ChatPanel() {
  const [messages, setMessages] = useState<LocalMessage[]>([]);
  const [input, setInput] = useState("");
  const [isStreaming, setIsStreaming] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const scrollerRef = useRef<HTMLDivElement>(null);
  const activeStreamIdRef = useRef<string | null>(null);
  const activeAssistantIdRef = useRef<string | null>(null);

  function claimStreamEvent(streamId: string) {
    if (!activeAssistantIdRef.current) return null;
    if (!activeStreamIdRef.current) {
      activeStreamIdRef.current = streamId;
    }
    return activeStreamIdRef.current === streamId ? activeAssistantIdRef.current : null;
  }

  useEffect(() => {
    let disposed = false;
    void getChatHistory()
      .then((history) => {
        if (disposed) return;
        setMessages(
          history
            .filter((msg) => msg.role === "user" || msg.role === "assistant")
            .map((msg, index) => ({ ...msg, id: `history-${index}` })),
        );
      })
      .catch((err: unknown) => setError(err instanceof Error ? err.message : String(err)));

    void getPendingMessage().then((pending) => {
      if (!disposed && pending) {
        appendPendingMessage(pending);
      }
    });

    return () => {
      disposed = true;
    };
  }, []);

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
        setMessages((current) =>
          current.map((msg) =>
            msg.id === assistantId
              ? { ...msg, pending: false, meta: emotionMeta(event.payload.meta.emotion) }
              : msg,
          ),
        );
        activeStreamIdRef.current = null;
        activeAssistantIdRef.current = null;
        setIsStreaming(false);
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

  async function handleSend() {
    const text = input.trim();
    if (!text || isStreaming) return;

    setError(null);
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
    await clearChatHistory();
    activeAssistantIdRef.current = null;
    activeStreamIdRef.current = null;
    setIsStreaming(false);
    setMessages([]);
  }

  return (
    <div className="flex-1 min-h-0 flex flex-col bg-paper/30">
      <div className="h-10 px-5 border-b border-paper-deep/20 flex items-center justify-between shrink-0">
        <div>
          <h2 className="text-[13px] font-display font-medium text-ink-soft">对话</h2>
          {error && <p className="text-[10px] text-red-400">{error}</p>}
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
            今天发生了什么，直接说就好。
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
                {msg.meta && <div className="mt-1 text-[10px] opacity-60">{msg.meta}</div>}
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
            placeholder="输入一句话，Enter 发送，Shift+Enter 换行"
            className="flex-1 min-h-[44px] max-h-[120px] text-[13px] leading-6 text-ink-soft placeholder:text-ink-ghost/60"
            disabled={isStreaming}
          />
          <button
            type="button"
            onClick={() => void handleSend()}
            disabled={!input.trim() || isStreaming}
            className="self-end h-8 px-4 rounded-lg bg-bamboo text-cloud text-[12px] disabled:opacity-40 disabled:cursor-not-allowed hover:bg-bamboo-light transition-colors"
          >
            发送
          </button>
        </div>
      </div>
    </div>
  );
}
