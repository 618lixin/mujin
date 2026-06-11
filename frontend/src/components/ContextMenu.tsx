import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { listen } from "@tauri-apps/api/event";
import { getConfig } from "../features/settings/api";
import type { AppConfig } from "../features/settings/types";
import { requestSurfaceAction } from "../features/windows/surfaceActions";
import { getPinboardContextMenuItems } from "../features/windows/tileContextMenu";

interface MenuState {
  x: number;
  y: number;
  hasSelection: boolean;
  type: "edit" | "pinboard";
}

export function ContextMenuProvider({ children }: { children: React.ReactNode }) {
  const { t } = useTranslation();
  const [menu, setMenu] = useState<MenuState | null>(null);
  const [menuClosing, setMenuClosing] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);
  const pinboardCtrlCloseRef = useRef(true);
  const pinboardContextMenuItems = useMemo(() => getPinboardContextMenuItems(t), [t]);

  useEffect(() => {
    getConfig()
      .then((c) => {
        pinboardCtrlCloseRef.current = c.tileCtrlClose ?? true;
      })
      .catch(() => {});
    const unlisten = listen<AppConfig>("config-changed", (event) => {
      pinboardCtrlCloseRef.current = event.payload.tileCtrlClose ?? true;
    });
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    function handleContextMenu(event: MouseEvent) {
      const target = event.target as HTMLElement;
      const isEditable =
        target.tagName === "TEXTAREA" || target.tagName === "INPUT" || target.isContentEditable;
      const pinboardTarget = target.closest<HTMLElement>('[data-context-menu="pinboard"]');

      if (!isEditable && !pinboardTarget) {
        event.preventDefault();
        return;
      }

      event.preventDefault();

      if (pinboardTarget && event.ctrlKey && pinboardCtrlCloseRef.current) {
        requestSurfaceAction("close");
        return;
      }
      const selection = window.getSelection()?.toString() || "";

      let x = event.clientX;
      let y = event.clientY;
      const menuWidth = 160;
      const menuHeight = pinboardTarget ? 150 : 170;
      if (x + menuWidth > window.innerWidth) x = window.innerWidth - menuWidth - 4;
      if (y + menuHeight > window.innerHeight) y = window.innerHeight - menuHeight - 4;

      if (pinboardTarget) {
        setMenuClosing(false);
        setMenu({
          x,
          y,
          hasSelection: false,
          type: "pinboard",
        });
        return;
      }

      setMenuClosing(false);
      setMenu({ x, y, hasSelection: selection.length > 0, type: "edit" });
    }

    function handleClick() {
      setMenuClosing(true);
    }

    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") setMenuClosing(true);
    }

    document.addEventListener("contextmenu", handleContextMenu);
    document.addEventListener("mousedown", handleClick);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("contextmenu", handleContextMenu);
      document.removeEventListener("mousedown", handleClick);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, []);

  useEffect(() => {
    if (!menuClosing || !menu) return;
    const timer = window.setTimeout(() => {
      setMenu(null);
      setMenuClosing(false);
    }, 150);
    return () => window.clearTimeout(timer);
  }, [menuClosing, menu]);

  const dismissMenu = useCallback(() => {
    setMenuClosing(true);
  }, []);

  const runCommand = (command: string) => {
    document.execCommand(command);
    dismissMenu();
  };

  const runSurfaceAction = (action: (typeof pinboardContextMenuItems)[number]["action"]) => {
    requestSurfaceAction(action);
    dismissMenu();
  };

  const items = useMemo(
    () =>
      menu
        ? menu.type === "pinboard"
          ? pinboardContextMenuItems.map((item) => ({
              ...item,
              shortcut: "",
              action: () => runSurfaceAction(item.action),
              disabled: false,
            }))
          : [
              {
                label: t("contextMenu.edit.cut", { defaultValue: "剪切" }),
                shortcut: "Ctrl+X",
                action: () => runCommand("cut"),
                disabled: !menu.hasSelection,
              },
              {
                label: t("contextMenu.edit.copy", { defaultValue: "复制" }),
                shortcut: "Ctrl+C",
                action: () => runCommand("copy"),
                disabled: !menu.hasSelection,
              },
              {
                label: t("contextMenu.edit.paste", { defaultValue: "粘贴" }),
                shortcut: "Ctrl+V",
                action: () => runCommand("paste"),
                disabled: false,
              },
              { separator: true as const },
              {
                label: t("contextMenu.edit.selectAll", { defaultValue: "全选" }),
                shortcut: "Ctrl+A",
                action: () => runCommand("selectAll"),
                disabled: false,
              },
            ]
        : [],
    [menu, runCommand, t, pinboardContextMenuItems],
  );

  return (
    <>
      {children}
      {menu && (
        <div
          ref={menuRef}
          className={`fixed z-[9999] min-w-[152px] py-1.5 bg-cloud/95 backdrop-blur-sm border border-paper-deep/50 rounded-lg overflow-hidden select-none ${menuClosing ? "animate-menu-exit" : "animate-menu-enter"}`}
          style={{
            left: menu.x,
            top: menu.y,
          }}
          onMouseDown={(event) => event.stopPropagation()}
        >
          {items.map((item, index) =>
            "separator" in item ? (
              <div key={index} className="mx-2 my-1 h-px bg-paper-deep/40" />
            ) : (
              <button
                key={item.label}
                onClick={() => void item.action()}
                disabled={item.disabled}
                className={`w-full flex items-center justify-between px-3 py-1.5 text-[12px] font-body transition-colors cursor-pointer disabled:text-ink-ghost/40 disabled:cursor-default disabled:hover:bg-transparent ${
                  "tone" in item && item.tone === "danger"
                    ? "text-red-400 hover:bg-danger-bg hover:text-red-500"
                    : "text-ink-soft hover:bg-bamboo-mist/60 hover:text-bamboo"
                }`}
              >
                <span>{item.label}</span>
                {item.shortcut && (
                  <span className="text-[10px] text-ink-ghost/60 font-mono ml-6">
                    {item.shortcut}
                  </span>
                )}
              </button>
            ),
          )}
        </div>
      )}
    </>
  );
}
