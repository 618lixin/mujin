import { useEffect, useMemo, useState } from "react";
import {
  generateDiary,
  getDiary,
  getDiaryList,
  regenerateDiary,
} from "../../features/api/diary";
import {
  generateLifeChapter,
  generateWeeklySummary,
  getLifeChapters,
  getWeeklySummaries,
  regenerateWeeklySummary,
  updateLifeChapter,
  updateWeeklySummary,
} from "../../features/api/growthReview";
import { getTopics } from "../../features/api/memory";
import {
  getGrowthLines,
  getObservations,
  getProjects,
} from "../../features/api/observations";
import type {
  DiaryEntry,
  GrowthLine,
  LifeChapterEntry,
  Observation,
  Project,
  Topic,
  WeeklySummaryEntry,
} from "../../features/api/types";
import { MarkdownPreview } from "../../features/markdown/MarkdownPreview";

type ReviewTab = "diary" | "weekly" | "chapter";

function today() {
  return new Date().toISOString().slice(0, 10);
}

function isoWeek(date = new Date()) {
  const utc = new Date(Date.UTC(date.getFullYear(), date.getMonth(), date.getDate()));
  const day = utc.getUTCDay() || 7;
  utc.setUTCDate(utc.getUTCDate() + 4 - day);
  const yearStart = new Date(Date.UTC(utc.getUTCFullYear(), 0, 1));
  const week = Math.ceil(((utc.getTime() - yearStart.getTime()) / 86400000 + 1) / 7);
  return { isoYear: utc.getUTCFullYear(), isoWeek: week };
}

function shortDate(value: string) {
  if (!value) return "--";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return `${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
}

function sourceTotal(counts?: object) {
  if (!counts) return 0;
  return Object.values(counts).reduce(
    (sum, value) => sum + (typeof value === "number" ? value : 0),
    0,
  );
}

function sourceSuffix(counts?: object) {
  const total = sourceTotal(counts);
  return total > 0 ? ` · ${total} 条来源` : "";
}

export function GrowthPanel() {
  const currentWeek = useMemo(() => isoWeek(), []);
  const [tab, setTab] = useState<ReviewTab>("diary");
  const [diaries, setDiaries] = useState<DiaryEntry[]>([]);
  const [selectedDiary, setSelectedDiary] = useState<DiaryEntry | null>(null);
  const [weeklySummaries, setWeeklySummaries] = useState<WeeklySummaryEntry[]>([]);
  const [selectedWeekly, setSelectedWeekly] = useState<WeeklySummaryEntry | null>(null);
  const [lifeChapters, setLifeChapters] = useState<LifeChapterEntry[]>([]);
  const [selectedChapter, setSelectedChapter] = useState<LifeChapterEntry | null>(null);
  const [observations, setObservations] = useState<Observation[]>([]);
  const [topics, setTopics] = useState<Topic[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [growthLines, setGrowthLines] = useState<GrowthLine[]>([]);
  const [diaryDate, setDiaryDate] = useState(today());
  const [weekYear, setWeekYear] = useState(String(currentWeek.isoYear));
  const [weekNumber, setWeekNumber] = useState(String(currentWeek.isoWeek));
  const [chapterStart, setChapterStart] = useState(today());
  const [chapterEnd, setChapterEnd] = useState(today());
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [editing, setEditing] = useState(false);
  const [editTitle, setEditTitle] = useState("");
  const [editContent, setEditContent] = useState("");

  async function refreshAll() {
    setError(null);
    const [
      diaryList,
      weeklyList,
      chapterList,
      obs,
      topicList,
      projectList,
      lineList,
    ] = await Promise.all([
      getDiaryList(60),
      getWeeklySummaries(),
      getLifeChapters(),
      getObservations(undefined, 80),
      getTopics(20),
      getProjects(undefined, 20),
      getGrowthLines(20),
    ]);
    setDiaries(diaryList);
    setWeeklySummaries(weeklyList);
    setLifeChapters(chapterList);
    setObservations(obs);
    setTopics(topicList);
    setProjects(projectList);
    setGrowthLines(lineList);
    setSelectedWeekly((current) =>
      current
        ? weeklyList.find((entry) => entry.noteId === current.noteId) ?? current
        : weeklyList[0] ?? null,
    );
    setSelectedChapter((current) =>
      current
        ? chapterList.find((entry) => entry.noteId === current.noteId) ?? current
        : chapterList[0] ?? null,
    );
  }

  useEffect(() => {
    let disposed = false;
    setLoading(true);
    refreshAll()
      .catch((err: unknown) => {
        if (!disposed) setError(err instanceof Error ? err.message : String(err));
      })
      .finally(() => {
        if (!disposed) setLoading(false);
      });
    return () => {
      disposed = true;
    };
  }, []);

  async function runAction(name: string, action: () => Promise<void>) {
    setBusy(name);
    setError(null);
    try {
      await action();
      await refreshAll();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(null);
    }
  }

  async function openDiary(date: string) {
    setBusy(`diary-${date}`);
    setError(null);
    try {
      setSelectedDiary(await getDiary(date));
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(null);
    }
  }

  useEffect(() => {
    setEditing(false);
    setEditTitle(tab === "chapter" ? selectedChapter?.title ?? "" : "");
    setEditContent(
      tab === "weekly"
        ? selectedWeekly?.content ?? ""
        : tab === "chapter"
          ? selectedChapter?.content ?? ""
          : "",
    );
  }, [tab, selectedWeekly?.noteId, selectedChapter?.noteId]);

  const detailContent =
    tab === "diary"
      ? selectedDiary?.content
      : tab === "weekly"
        ? selectedWeekly?.content
        : selectedChapter?.content;
  const canEdit =
    (tab === "weekly" && selectedWeekly !== null) ||
    (tab === "chapter" && selectedChapter !== null);
  const dirty =
    tab === "weekly"
      ? editContent !== (selectedWeekly?.content ?? "")
      : tab === "chapter"
        ? editTitle !== (selectedChapter?.title ?? "") ||
          editContent !== (selectedChapter?.content ?? "")
        : false;

  function startEdit() {
    if (!canEdit) return;
    setError(null);
    setEditTitle(tab === "chapter" ? selectedChapter?.title ?? "" : "");
    setEditContent(
      tab === "weekly" ? selectedWeekly?.content ?? "" : selectedChapter?.content ?? "",
    );
    setEditing(true);
  }

  function cancelEdit() {
    setError(null);
    setEditTitle(tab === "chapter" ? selectedChapter?.title ?? "" : "");
    setEditContent(
      tab === "weekly" ? selectedWeekly?.content ?? "" : selectedChapter?.content ?? "",
    );
    setEditing(false);
  }

  async function saveEdit() {
    if (tab === "weekly" && selectedWeekly) {
      await runAction("save-week", async () => {
        const result = await updateWeeklySummary(
          selectedWeekly.isoYear,
          selectedWeekly.isoWeek,
          editContent,
        );
        const updated: WeeklySummaryEntry = {
          ...selectedWeekly,
          ...result,
          sourceCounts: selectedWeekly.sourceCounts,
          createdAt: selectedWeekly.createdAt,
        };
        setSelectedWeekly(updated);
        setWeeklySummaries((items) =>
          items.map((item) => (item.noteId === updated.noteId ? updated : item)),
        );
        setEditing(false);
      });
      return;
    }
    if (tab === "chapter" && selectedChapter) {
      await runAction("save-chapter", async () => {
        const result = await updateLifeChapter(selectedChapter.noteId, editTitle, editContent);
        const updated: LifeChapterEntry = {
          ...selectedChapter,
          ...result,
          sourceCounts: selectedChapter.sourceCounts,
          createdAt: selectedChapter.createdAt,
        };
        setSelectedChapter(updated);
        setLifeChapters((items) =>
          items.map((item) => (item.noteId === updated.noteId ? updated : item)),
        );
        setEditing(false);
      });
    }
  }

  if (loading) {
    return (
      <div className="flex-1 flex items-center justify-center text-[13px] text-ink-ghost">
        正在加载成长回顾...
      </div>
    );
  }

  return (
    <div className="flex-1 min-h-0 overflow-y-auto bg-paper/30 p-5">
      <div className="grid grid-cols-[minmax(300px,0.8fr)_minmax(420px,1.2fr)] gap-5">
        <section className="min-h-0 space-y-4">
          <div className="flex rounded-lg border border-paper-deep/40 bg-cloud/80 p-1">
            {(["diary", "weekly", "chapter"] as const).map((item) => (
              <button
                key={item}
                onClick={() => setTab(item)}
                className={`flex-1 rounded-md px-3 py-2 text-[12px] transition-all ${
                  tab === item ? "bg-bamboo text-cloud" : "text-ink-faint hover:bg-paper-warm"
                }`}
              >
                {item === "diary" ? "日记" : item === "weekly" ? "周总结" : "人生章节"}
              </button>
            ))}
          </div>

          {error && (
            <div className="rounded-lg border border-red-200 bg-danger-bg px-3 py-2 text-[12px] text-red-500">
              {error}
            </div>
          )}

          {tab === "diary" && (
            <section className="rounded-lg border border-paper-deep/35 bg-cloud/80 p-4">
              <div className="flex gap-2">
                <input
                  type="date"
                  value={diaryDate}
                  onChange={(event) => setDiaryDate(event.target.value)}
                  className="min-w-0 flex-1 rounded-md bg-paper/70 px-3 py-2 text-[12px] text-ink-soft"
                />
                <button
                  onClick={() =>
                    runAction("generate-diary", async () => {
                      const result = await generateDiary(diaryDate);
                      setSelectedDiary({
                        date: result.date,
                        noteId: result.noteId,
                        title: result.title,
                        content: result.content,
                        createdAt: "",
                        updatedAt: "",
                      });
                    })
                  }
                  disabled={busy !== null}
                  className="rounded-md bg-bamboo px-3 py-2 text-[12px] text-cloud disabled:opacity-50"
                >
                  生成
                </button>
              </div>
              <div className="mt-3 max-h-[360px] space-y-2 overflow-y-auto">
                {diaries.length === 0 ? (
                  <p className="text-[12px] text-ink-ghost">还没有日记。</p>
                ) : (
                  diaries.map((entry) => (
                    <button
                      key={entry.noteId}
                      onClick={() => void openDiary(entry.date)}
                      className="w-full rounded-md bg-paper/55 px-3 py-2 text-left text-[12px] text-ink-soft hover:bg-bamboo-mist"
                    >
                      {entry.date}
                    </button>
                  ))
                )}
              </div>
            </section>
          )}

          {tab === "weekly" && (
            <section className="rounded-lg border border-paper-deep/35 bg-cloud/80 p-4">
              <div className="grid grid-cols-[1fr_0.8fr_auto] gap-2">
                <input
                  value={weekYear}
                  onChange={(event) => setWeekYear(event.target.value)}
                  className="rounded-md bg-paper/70 px-3 py-2 text-[12px] text-ink-soft"
                />
                <input
                  value={weekNumber}
                  onChange={(event) => setWeekNumber(event.target.value)}
                  className="rounded-md bg-paper/70 px-3 py-2 text-[12px] text-ink-soft"
                />
                <button
                  onClick={() =>
                    runAction("generate-week", async () => {
                      const result = await generateWeeklySummary(Number(weekYear), Number(weekNumber));
                      setSelectedWeekly({
                        ...result,
                        createdAt: "",
                        updatedAt: "",
                      });
                    })
                  }
                  disabled={busy !== null}
                  className="rounded-md bg-bamboo px-3 py-2 text-[12px] text-cloud disabled:opacity-50"
                >
                  生成
                </button>
              </div>
              <div className="mt-3 max-h-[360px] space-y-2 overflow-y-auto">
                {weeklySummaries.length === 0 ? (
                  <p className="text-[12px] text-ink-ghost">还没有周总结。</p>
                ) : (
                  weeklySummaries.map((entry) => (
                    <button
                      key={entry.noteId}
                      onClick={() => setSelectedWeekly(entry)}
                      className="w-full rounded-md bg-paper/55 px-3 py-2 text-left hover:bg-bamboo-mist"
                    >
                      <div className="text-[12px] text-ink-soft">{entry.title}</div>
                      <div className="text-[10px] text-ink-ghost">{entry.weekDisplayRange}</div>
                    </button>
                  ))
                )}
              </div>
            </section>
          )}

          {tab === "chapter" && (
            <section className="rounded-lg border border-paper-deep/35 bg-cloud/80 p-4">
              <div className="grid grid-cols-2 gap-2">
                <input
                  type="date"
                  value={chapterStart}
                  onChange={(event) => setChapterStart(event.target.value)}
                  className="rounded-md bg-paper/70 px-3 py-2 text-[12px] text-ink-soft"
                />
                <input
                  type="date"
                  value={chapterEnd}
                  onChange={(event) => setChapterEnd(event.target.value)}
                  className="rounded-md bg-paper/70 px-3 py-2 text-[12px] text-ink-soft"
                />
              </div>
              <button
                onClick={() =>
                  runAction("generate-chapter", async () => {
                    const result = await generateLifeChapter(chapterStart, chapterEnd);
                    setSelectedChapter({
                      ...result,
                      createdAt: "",
                      updatedAt: "",
                    });
                  })
                }
                disabled={busy !== null}
                className="mt-2 w-full rounded-md bg-bamboo px-3 py-2 text-[12px] text-cloud disabled:opacity-50"
              >
                生成章节
              </button>
              <div className="mt-3 max-h-[330px] space-y-2 overflow-y-auto">
                {lifeChapters.length === 0 ? (
                  <p className="text-[12px] text-ink-ghost">还没有人生章节。</p>
                ) : (
                  lifeChapters.map((entry) => (
                    <button
                      key={entry.noteId}
                      onClick={() => setSelectedChapter(entry)}
                      className="w-full rounded-md bg-paper/55 px-3 py-2 text-left hover:bg-bamboo-mist"
                    >
                      <div className="text-[12px] text-ink-soft">{entry.title}</div>
                      <div className="text-[10px] text-ink-ghost">
                        {entry.startDate || "--"} ~ {entry.endDate || "--"}
                      </div>
                    </button>
                  ))
                )}
              </div>
            </section>
          )}
        </section>

        <section className="min-h-[520px] rounded-lg border border-paper-deep/35 bg-cloud/85 p-4">
          <div className="mb-3 flex items-center justify-between gap-3">
            <div className="min-w-0">
              <h2 className="truncate text-[14px] font-medium text-ink-soft">
                {tab === "diary"
                  ? selectedDiary?.title ?? "日记回顾"
                  : tab === "weekly"
                    ? selectedWeekly?.title ?? "周总结"
                    : selectedChapter?.title ?? "人生章节"}
              </h2>
              <p className="text-[10px] text-ink-ghost">
                {tab === "weekly" && selectedWeekly
                  ? `${selectedWeekly.weekDisplayRange}${sourceSuffix(selectedWeekly.sourceCounts)}`
                  : tab === "chapter" && selectedChapter
                    ? `${selectedChapter.startDate || "--"} ~ ${selectedChapter.endDate || "--"}${sourceSuffix(selectedChapter.sourceCounts)}`
                    : selectedDiary
                      ? selectedDiary.date
                      : "选择或生成一项回顾"}
              </p>
            </div>
            <div className="flex shrink-0 items-center gap-2">
            {editing && dirty && (
              <span className="text-[10px] text-ink-ghost">未保存</span>
            )}
            {tab === "diary" && selectedDiary && (
              <button
                onClick={() =>
                  runAction("regenerate-diary", async () => {
                    const result = await regenerateDiary(selectedDiary.date);
                    setSelectedDiary({
                      ...selectedDiary,
                      title: result.title,
                      content: result.content,
                    });
                  })
                }
                disabled={busy !== null}
                className="rounded-md border border-paper-deep px-3 py-1.5 text-[12px] text-ink-faint hover:bg-paper-warm disabled:opacity-50"
              >
                重新生成
              </button>
            )}
            {tab === "weekly" && selectedWeekly && !editing && (
              <button
                onClick={() =>
                  runAction("regenerate-week", async () => {
                    const result = await regenerateWeeklySummary(
                      selectedWeekly.isoYear,
                      selectedWeekly.isoWeek,
                    );
                    setSelectedWeekly({ ...result, createdAt: "", updatedAt: "" });
                  })
                }
                disabled={busy !== null}
                className="rounded-md border border-paper-deep px-3 py-1.5 text-[12px] text-ink-faint hover:bg-paper-warm disabled:opacity-50"
              >
                重新生成
              </button>
            )}
            {canEdit && !editing && (
              <button
                onClick={startEdit}
                disabled={busy !== null}
                className="rounded-md border border-paper-deep px-3 py-1.5 text-[12px] text-ink-faint hover:bg-paper-warm disabled:opacity-50"
              >
                编辑
              </button>
            )}
            {editing && (
              <>
                <button
                  onClick={cancelEdit}
                  disabled={busy !== null}
                  className="rounded-md border border-paper-deep px-3 py-1.5 text-[12px] text-ink-faint hover:bg-paper-warm disabled:opacity-50"
                >
                  取消
                </button>
                <button
                  onClick={() => void saveEdit()}
                  disabled={busy !== null || !dirty}
                  className="rounded-md bg-bamboo px-3 py-1.5 text-[12px] text-cloud disabled:opacity-50"
                >
                  保存
                </button>
              </>
            )}
            </div>
          </div>
          <div className="h-[470px] overflow-y-auto rounded-md bg-paper/45 px-4 py-3">
            {editing ? (
              <div className="space-y-3">
                {tab === "chapter" && (
                  <input
                    value={editTitle}
                    onChange={(event) => setEditTitle(event.target.value)}
                    className="w-full rounded-md border border-paper-deep/40 bg-cloud px-3 py-2 text-[13px] text-ink-soft outline-none focus:border-bamboo/60"
                  />
                )}
                <textarea
                  value={editContent}
                  onChange={(event) => setEditContent(event.target.value)}
                  className="min-h-[410px] w-full resize-none rounded-md border border-paper-deep/40 bg-cloud px-3 py-3 font-mono text-[13px] leading-6 text-ink-soft outline-none focus:border-bamboo/60"
                />
              </div>
            ) : detailContent ? (
              <MarkdownPreview content={detailContent} fontSize={14} />
            ) : (
              <div className="flex h-full items-center justify-center text-[12px] text-ink-ghost">
                暂未选择内容。
              </div>
            )}
          </div>
        </section>
      </div>

      <div className="mt-5 grid grid-cols-[minmax(360px,1.2fr)_repeat(3,minmax(180px,0.8fr))] gap-4">
        <section className="rounded-lg border border-paper-deep/35 bg-cloud/80 p-4">
          <h3 className="mb-3 text-[12px] font-medium text-ink-faint">定性观察</h3>
          <div className="max-h-[220px] space-y-2 overflow-y-auto">
            {observations.length === 0 ? (
              <p className="text-[12px] text-ink-ghost">还没有观察记录。</p>
            ) : (
              observations.map((item) => (
                <article key={item.id} className="rounded-md bg-paper/55 px-3 py-2">
                  <div className="flex gap-2 text-[10px] text-ink-ghost">
                    <span>{shortDate(item.date)}</span>
                    {item.category && <span>{item.category}</span>}
                    {item.source && <span>{item.source}</span>}
                  </div>
                  <p className="mt-1 text-[12px] leading-6 text-ink-soft">{item.content}</p>
                </article>
              ))
            )}
          </div>
        </section>

        <MiniList title="主题" items={topics.map((topic) => `${topic.name} · ${topic.mentionCount}`)} />
        <MiniList title="项目" items={projects.map((project) => `${project.title} · ${project.status}`)} />
        <MiniList title="成长线" items={growthLines.map((line) => `${line.dimension} · ${line.records.length}`)} />
      </div>
    </div>
  );
}

function MiniList({ title, items }: { title: string; items: string[] }) {
  return (
    <section className="rounded-lg border border-paper-deep/35 bg-cloud/80 p-4">
      <h3 className="mb-3 text-[12px] font-medium text-ink-faint">{title}</h3>
      <div className="max-h-[220px] space-y-2 overflow-y-auto">
        {items.length === 0 ? (
          <p className="text-[12px] text-ink-ghost">暂无内容</p>
        ) : (
          items.map((item) => (
            <div key={item} className="truncate rounded-md bg-paper/55 px-3 py-2 text-[12px] text-ink-soft">
              {item}
            </div>
          ))
        )}
      </div>
    </section>
  );
}
