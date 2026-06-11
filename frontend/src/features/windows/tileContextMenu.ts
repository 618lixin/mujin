import { t, type TFunction } from "i18next";
import type { NoteSurfaceAction } from "./surfaceActions";

export interface PinboardContextMenuItem {
  action: NoteSurfaceAction;
  label: string;
  tone?: "danger";
}

export function getPinboardContextMenuItems(translate: TFunction = t): PinboardContextMenuItem[] {
  return [
    {
      action: "copy",
      label: translate("contextMenu.pinboard.copy", { defaultValue: "复制" }),
    },
    {
      action: "save",
      label: translate("contextMenu.pinboard.save", { defaultValue: "保存" }),
    },
    {
      action: "switchToPad",
      label: translate("contextMenu.pinboard.switchToPad", { defaultValue: "转为小窗" }),
    },
    {
      action: "close",
      label: translate("contextMenu.pinboard.close", { defaultValue: "取消钉屏" }),
      tone: "danger",
    },
  ];
}
