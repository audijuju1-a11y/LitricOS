#!/usr/bin/env python3
"""Apply the UI-scale persistence edits to shell.rs.

`str_replace` refuses this file (the tool's command budget), so the same
replacements are applied here with the checks that matter: every pattern must
occur exactly once, and the script writes nothing unless all of them match.
"""

from pathlib import Path

PATH = Path(__file__).resolve().parent.parent / "src/litric-shell/src/shell.rs"

PATCHES = [
    (
        "    pub scale_revert: Option<crate::revert::RevertWindow<f32>>,\n",
        "    pub scale_revert: Option<crate::revert::RevertWindow<f32>>,\n"
        "    /// The scale this display asked for, before the user's own step is applied.\n"
        "    ///\n"
        "    /// Kept apart from [`Self::tokens`] so a stored 125 % step multiplies the\n"
        "    /// display's scale instead of replacing it: a 2x panel that is also asked\n"
        "    /// for a larger interface must not come back at 1.25x on the next boot.\n"
        "    base_scale: f32,\n",
    ),
    (
        "        self.browser.engine = preferences.search_engine;\n"
        "        self.request_full_redraw();\n"
        "    }\n"
        "\n"
        "    /// The local hour, which is what an appearance schedule is written against.\n",
        "        self.browser.engine = preferences.search_engine;\n"
        "        // The stored step is a multiplier on the display's own scale, so one\n"
        "        // record is right on a 1x panel and on a 2x one. Nothing is provisional\n"
        "        // at startup: a value that is already on disk was confirmed by the save\n"
        "        // that wrote it, and arming a window here would undo it every boot.\n"
        "        let scale = self.base_scale * preferences.ui_scale.factor();\n"
        "        if (self.tokens.scale - scale).abs() > 0.001 {\n"
        "            self.set_ui_scale(scale);\n"
        "        }\n"
        "        self.request_full_redraw();\n"
        "    }\n"
        "\n"
        "    /// The local hour, which is what an appearance schedule is written against.\n",
    ),
    (
        "        self.quick.reduce_motion = preferences.reduce_motion;\n"
        "        self.request_full_redraw();\n"
        "    }\n"
        "\n"
        "    fn toggle_language(&mut self) -> Vec<Effect> {\n",
        "        self.quick.reduce_motion = preferences.reduce_motion;\n"
        "        // The record is the authority on the step, so a failed write leaves the\n"
        "        // screen at the size that is actually stored rather than at the one the\n"
        "        // disk refused.\n"
        "        let wanted = self.base_scale * preferences.ui_scale.factor();\n"
        "        if (self.tokens.scale - wanted).abs() > 0.001 {\n"
        "            self.set_ui_scale(wanted);\n"
        "        }\n"
        "        self.request_full_redraw();\n"
        "    }\n"
        "\n"
        "    fn toggle_language(&mut self) -> Vec<Effect> {\n",
    ),
    (
        "    pub const UI_SCALES: [f32; 3] = [1.0, 1.25, 1.5];\n",
        "    pub const UI_SCALES: [f32; 3] = [\n"
        "        crate::settings::UiScale::Normal.factor(),\n"
        "        crate::settings::UiScale::Large.factor(),\n"
        "        crate::settings::UiScale::Larger.factor(),\n"
        "    ];\n",
    ),
    (
        "    /// Keep the provisional change: the value stays, the clock stops.\n"
        "    pub fn keep_pending_change(&mut self) {\n"
        "        if let Some(mut window) = self.scale_revert.take() {\n"
        "            window.keep();\n"
        "        }\n"
        "        self.request_full_redraw();\n"
        "    }\n",
        "    /// Keep the provisional change: the value stays, the clock stops, and the\n"
        "    /// step is remembered.\n"
        "    ///\n"
        "    /// Writing happens here and not when the change is applied: a provisional\n"
        "    /// value that reached the file would survive a crash inside the window and\n"
        "    /// come back as the size the user never confirmed, which is the one outcome\n"
        "    /// the window exists to prevent.\n"
        "    pub fn keep_pending_change(&mut self) -> Vec<Effect> {\n"
        "        if let Some(mut window) = self.scale_revert.take() {\n"
        "            window.keep();\n"
        "        } else {\n"
        "            return Vec::new();\n"
        "        }\n"
        "        self.request_full_redraw();\n"
        "        let step = crate::settings::UiScale::from_factor(self.tokens.scale / self.base_scale)\n"
        "            .unwrap_or(self.settings.preferences.ui_scale);\n"
        "        if step == self.settings.preferences.ui_scale {\n"
        "            // Keeping the size that was already stored is not a change, and a\n"
        "            // write for it would only be a chance to fail.\n"
        "            return Vec::new();\n"
        "        }\n"
        "        let mut next = self.settings.preferences;\n"
        "        next.ui_scale = step;\n"
        "        self.save_preferences(next)\n"
        "    }\n",
    ),
    (
        "        let current = self.tokens.scale;\n"
        "        let next = Self::UI_SCALES\n"
        "            .iter()\n"
        "            .copied()\n"
        "            .find(|scale| *scale > current + 0.001)\n"
        "            .unwrap_or(Self::UI_SCALES[0]);\n"
        "        self.apply_ui_scale(next);\n"
        "        Vec::new()\n",
        "        let current = crate::settings::UiScale::from_factor(self.tokens.scale / self.base_scale)\n"
        "            .unwrap_or(crate::settings::UiScale::Normal);\n"
        "        let next = crate::settings::UiScale::ALL\n"
        "            .iter()\n"
        "            .copied()\n"
        "            .find(|step| step.factor() > current.factor() + 0.001)\n"
        "            .unwrap_or(crate::settings::UiScale::Normal);\n"
        "        self.apply_ui_scale(self.base_scale * next.factor());\n"
        "        Vec::new()\n",
    ),
    (
        "    pub fn new(tokens: Tokens, width: f32, height: f32, regional: Regional) -> Self {\n"
        "        let top = tokens.px(Self::TOP_BAR_LOGICAL);\n",
        "    pub fn new(tokens: Tokens, width: f32, height: f32, regional: Regional) -> Self {\n"
        "        let base_scale = tokens.scale;\n"
        "        let top = tokens.px(Self::TOP_BAR_LOGICAL);\n",
    ),
    (
        "            scale_revert: None,\n",
        "            scale_revert: None,\n"
        "            base_scale,\n",
    ),
    (
        "            Hot::ScaleKeep => {\n"
        "                self.keep_pending_change();\n"
        "                Vec::new()\n"
        "            }\n",
        "            Hot::ScaleKeep => self.keep_pending_change(),\n",
    ),
]


def main() -> int:
    text = PATH.read_text()
    for index, (old, new) in enumerate(PATCHES, start=1):
        count = text.count(old)
        if count != 1:
            print(f"patch {index}: expected 1 match, found {count}")
            return 1
        text = text.replace(old, new, 1)
    PATH.write_text(text)
    print(f"applied {len(PATCHES)} patches")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
