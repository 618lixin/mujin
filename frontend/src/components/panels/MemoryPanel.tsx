import { useEffect, useMemo, useState } from "react";
import {
  deleteEvent,
  getCoreMemory,
  getEvents,
  getTopics,
  maintainMemory,
  patchCoreMemory,
} from "../../features/api/memory";
import type { CoreMemoryResponse, EventMemory, Topic } from "../../features/api/types";

function pct(value: number) {
  return `${Math.round(value * 100)}%`;
}

function formatDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
}

function MemoryCard({
  title,
  stats,
  onSave,
}: {
  title: string;
  stats: CoreMemoryResponse["profile"];
  onSave: (content: string) => Promise<void>;
}) {
  const [draft, setDraft] = useState(stats.content);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    setDraft(stats.content);
  }, [stats.content]);

  return (
    <section className="rounded-lg border border-paper-deep/35 bg-cloud/80 p-4">
      <div className="flex items-center justify-between gap-3">
        <h3 className="text-[13px] font-medium text-ink-soft">{title}</h3>
        <span className={`text-[10px] ${stats.nearLimit ? "text-red-400" : "text-ink-ghost"}`}>
          {stats.chars}/{stats.maxChars} · {pct(stats.pct)}
        </span>
      </div>
      <div className="mt-2 h-1 rounded-full bg-paper-deep/50 overflow-hidden">
        <div className="h-full bg-bamboo" style={{ width: pct(Math.min(stats.pct, 1)) }} />
      </div>
      <textarea
        value={draft}
        onChange={(event) => setDraft(event.target.value)}
        rows={6}
        className="mt-3 w-full rounded-lg border border-paper-deep/35 bg-paper/45 px-3 py-2 text-[12px] leading-6 text-ink-soft"
        placeholder="还没有内容"
      />
      <div className="mt-3 flex justify-end">
        <button
          type="button"
          onClick={() => {
            setSaving(true);
            void onSave(draft).finally(() => setSaving(false));
          }}
          disabled={saving || draft === stats.content}
          className="h-7 px-3 rounded-lg bg-bamboo text-cloud text-[11px] disabled:opacity-40"
        >
          {saving ? "保存中" : "保存"}
        </button>
      </div>
    </section>
  );
}

export function MemoryPanel() {
  const [coreMemory, setCoreMemory] = useState<CoreMemoryResponse | null>(null);
  const [events, setEvents] = useState<EventMemory[]>([]);
  const [topics, setTopics] = useState<Topic[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const importantEvents = useMemo(
    () => events.filter((event) => event.importance >= 0.6 || event.strength >= 0.4),
    [events],
  );

  async function load() {
    setLoading(true);
    setError(null);
    try {
      const [memory, eventList, topicList] = await Promise.all([
        getCoreMemory(),
        getEvents({ limit: 50 }),
        getTopics(50),
      ]);
      setCoreMemory(memory);
      setEvents(eventList);
      setTopics(topicList);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void load();
  }, []);

  async function saveMemory(target: "profile" | "notes", oldText: string, content: string) {
    const next = await patchCoreMemory({
      action: oldText ? "replace" : "add",
      target,
      oldText: oldText || undefined,
      content,
    });
    setCoreMemory(next);
  }

  async function handleDeleteEvent(id: string) {
    await deleteEvent(id);
    setEvents((current) => current.filter((event) => event.id !== id));
  }

  async function handleMaintain() {
    await maintainMemory();
    await load();
  }

  if (loading && !coreMemory) {
    return <div className="flex-1 flex items-center justify-center text-[13px] text-ink-ghost">加载记忆中...</div>;
  }

  return (
    <div className="flex-1 min-h-0 flex bg-paper/30">
      <div className="w-[42%] min-w-[320px] border-r border-paper-deep/25 overflow-y-auto p-5 space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-[13px] font-display font-medium text-ink-soft">核心记忆</h2>
          <button
            type="button"
            onClick={() => void handleMaintain()}
            className="h-7 px-3 rounded-lg text-[11px] text-ink-ghost hover:text-bamboo hover:bg-bamboo-mist/50"
          >
            维护
          </button>
        </div>
        {error && <p className="text-[11px] text-red-400">{error}</p>}
        {coreMemory && (
          <>
            <MemoryCard
              title="用户画像"
              stats={coreMemory.profile}
              onSave={(content) => saveMemory("profile", coreMemory.profile.content, content)}
            />
            <MemoryCard
              title="AI 笔记"
              stats={coreMemory.notes}
              onSave={(content) => saveMemory("notes", coreMemory.notes.content, content)}
            />
          </>
        )}
      </div>

      <div className="flex-1 min-w-0 overflow-y-auto p-5 space-y-5">
        <section>
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-[12px] font-medium text-ink-faint">事件记忆</h3>
            <span className="text-[10px] text-ink-ghost">{importantEvents.length}/{events.length}</span>
          </div>
          <div className="space-y-2">
            {importantEvents.length === 0 ? (
              <div className="rounded-lg border border-paper-deep/30 bg-cloud/60 p-4 text-[12px] text-ink-ghost">
                暂无事件沉淀。
              </div>
            ) : (
              importantEvents.map((event) => (
                <article
                  key={event.id}
                  className="rounded-lg border border-paper-deep/35 bg-cloud/80 px-3 py-2"
                >
                  <div className="flex items-start justify-between gap-3">
                    <p className="text-[13px] leading-6 text-ink-soft">{event.content}</p>
                    <button
                      type="button"
                      onClick={() => void handleDeleteEvent(event.id)}
                      className="text-[11px] text-ink-ghost hover:text-red-500 shrink-0"
                    >
                      删除
                    </button>
                  </div>
                  <div className="mt-1 flex flex-wrap items-center gap-2 text-[10px] text-ink-ghost">
                    <span>{formatDate(event.createdAt)}</span>
                    <span>重要度 {Math.round(event.importance * 100)}%</span>
                    <span>强度 {Math.round(event.strength * 100)}%</span>
                    {event.eventType && <span>{event.eventType}</span>}
                    {event.emotions.map((emotion) => (
                      <span key={emotion} className="rounded bg-bamboo-mist px-1.5 text-bamboo">
                        {emotion}
                      </span>
                    ))}
                  </div>
                </article>
              ))
            )}
          </div>
        </section>

        <section>
          <h3 className="text-[12px] font-medium text-ink-faint mb-3">主题</h3>
          <div className="grid grid-cols-2 gap-2">
            {topics.length === 0 ? (
              <div className="col-span-2 rounded-lg border border-paper-deep/30 bg-cloud/60 p-4 text-[12px] text-ink-ghost">
                暂无主题。
              </div>
            ) : (
              topics.map((topic) => (
                <div key={topic.id} className="rounded-lg border border-paper-deep/35 bg-cloud/75 p-3">
                  <div className="text-[13px] text-ink-soft truncate">{topic.name}</div>
                  <div className="mt-1 text-[10px] text-ink-ghost">{topic.mentionCount} 次提及</div>
                </div>
              ))
            )}
          </div>
        </section>
      </div>
    </div>
  );
}
