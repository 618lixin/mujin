import { useEffect, useState } from "react";
import { getTopics } from "../../features/api/memory";
import { getGrowthLines, getObservations, getProjects } from "../../features/api/observations";
import type { GrowthLine, Observation, Project, Topic } from "../../features/api/types";

function shortDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return `${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
}

export function GrowthPanel() {
  const [observations, setObservations] = useState<Observation[]>([]);
  const [topics, setTopics] = useState<Topic[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [growthLines, setGrowthLines] = useState<GrowthLine[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let disposed = false;
    setLoading(true);
    void Promise.all([
      getObservations(undefined, 80),
      getTopics(20),
      getProjects(undefined, 20),
      getGrowthLines(20),
    ])
      .then(([obs, topicList, projectList, lineList]) => {
        if (disposed) return;
        setObservations(obs);
        setTopics(topicList);
        setProjects(projectList);
        setGrowthLines(lineList);
      })
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

  if (loading) {
    return <div className="flex-1 flex items-center justify-center text-[13px] text-ink-ghost">加载成长档案中...</div>;
  }

  return (
    <div className="flex-1 min-h-0 overflow-y-auto bg-paper/30 p-5">
      <div className="grid grid-cols-[minmax(360px,1.4fr)_minmax(280px,0.9fr)] gap-5">
        <section className="space-y-3">
          <div>
            <h2 className="text-[13px] font-display font-medium text-ink-soft">成长时间线</h2>
            {error && <p className="mt-1 text-[11px] text-red-400">{error}</p>}
          </div>
          <div className="relative pl-5">
            <div className="absolute left-[5px] top-2 bottom-2 w-px bg-paper-deep" />
            {observations.length === 0 ? (
              <div className="rounded-lg border border-paper-deep/30 bg-cloud/65 p-4 text-[12px] text-ink-ghost">
                暂无观察记录。后续反思会把长期变化放到这里。
              </div>
            ) : (
              observations.map((item) => (
                <article key={item.id} className="relative pb-5">
                  <div className="absolute -left-[19px] top-1.5 w-2.5 h-2.5 rounded-full border border-bamboo bg-cloud" />
                  <div className="rounded-lg border border-paper-deep/35 bg-cloud/80 px-4 py-3">
                    <div className="flex items-center gap-2 text-[10px] text-ink-ghost">
                      <span>{shortDate(item.date)}</span>
                      {item.category && <span className="rounded bg-bamboo-mist px-1.5 text-bamboo">{item.category}</span>}
                      {item.source && <span>{item.source}</span>}
                    </div>
                    <p className="mt-1 text-[13px] leading-7 text-ink-soft">{item.content}</p>
                  </div>
                </article>
              ))
            )}
          </div>
        </section>

        <aside className="space-y-4">
          <section className="rounded-lg border border-paper-deep/35 bg-cloud/80 p-4">
            <h3 className="text-[12px] font-medium text-ink-faint mb-3">主题热度</h3>
            <div className="space-y-2">
              {topics.length === 0 ? (
                <p className="text-[12px] text-ink-ghost">暂无主题。</p>
              ) : (
                topics.map((topic) => (
                  <div key={topic.id}>
                    <div className="flex justify-between text-[12px] text-ink-soft">
                      <span className="truncate">{topic.name}</span>
                      <span className="text-ink-ghost">{topic.mentionCount}</span>
                    </div>
                    <div className="mt-1 h-1 rounded-full bg-paper-deep/50 overflow-hidden">
                      <div
                        className="h-full bg-bamboo"
                        style={{ width: `${Math.min(100, topic.mentionCount * 12)}%` }}
                      />
                    </div>
                  </div>
                ))
              )}
            </div>
          </section>

          <section className="rounded-lg border border-paper-deep/35 bg-cloud/80 p-4">
            <h3 className="text-[12px] font-medium text-ink-faint mb-3">项目档案</h3>
            <div className="space-y-2">
              {projects.length === 0 ? (
                <p className="text-[12px] text-ink-ghost">暂无项目。</p>
              ) : (
                projects.map((project) => (
                  <div key={project.id} className="rounded-md bg-paper/55 px-3 py-2">
                    <div className="flex justify-between gap-2">
                      <span className="text-[12px] text-ink-soft truncate">{project.title}</span>
                      <span className="text-[10px] text-ink-ghost">{project.status}</span>
                    </div>
                    {project.summary && (
                      <p className="mt-1 text-[11px] leading-5 text-ink-faint line-clamp-2">
                        {project.summary}
                      </p>
                    )}
                  </div>
                ))
              )}
            </div>
          </section>

          <section className="rounded-lg border border-paper-deep/35 bg-cloud/80 p-4">
            <h3 className="text-[12px] font-medium text-ink-faint mb-3">成长线</h3>
            <div className="space-y-2">
              {growthLines.length === 0 ? (
                <p className="text-[12px] text-ink-ghost">暂无成长线。</p>
              ) : (
                growthLines.map((line) => (
                  <div key={line.id} className="flex items-center justify-between rounded-md bg-paper/55 px-3 py-2">
                    <span className="text-[12px] text-ink-soft truncate">{line.dimension}</span>
                    <span className="text-[10px] text-ink-ghost">{line.records.length} 条</span>
                  </div>
                ))
              )}
            </div>
          </section>
        </aside>
      </div>
    </div>
  );
}
