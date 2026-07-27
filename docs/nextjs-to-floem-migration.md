You are executing Phase 1 of the Next.js -> Floem Rust migration: Token & Style Registration.

Do NOT modify any files inside `src/ui/` during this step. Your objective is to establish the single source of truth for design tokens, style classes, and global stylesheets in native Floem.

Do NOT modify the mockup directory - it is the current source of truth for the migration that we wish to convert into a Floem UI.

### Step 1: Read Source Tokens

Parse `mockup/app/globals.css`. Note all values defined under `@theme inline` and `@layer components`.

### Step 2: Migrate The Design Tokens to `themes/base.toml` and `src/style/tokens.rs`

The goal is to migrate all token names and token types into types within the `tokens.rs` file, and implement their values and naming within the `base.toml` theme file so that it gets parsed correctly. The final result of these two files should match the current state of globals.css exactly, and the current state can be overwritten / tokens that dont match it can be discarded.

### Step 3: Implement Style Classes in `src/style/style.rs`

Implement all the necessary style classes from globals.css using Floem's `style_class!` macro to define typed class identifiers for every CSS component class:

- Cuelist/Grid: `CueRowGrid`, `CueRowGridPlaying`, `CueRowGridSelected`, `CuelistHeader`, `LabelDragHandle`, `BtnAddCueEnd`
- Buttons & Controls: `BtnPunchDown`, `BtnPanic`, `BtnGo`, `BtnIconSm`, `FieldInput`, `FieldTextarea`, `DeviceChip`, `DeviceDot`
- Status & LED Graphics: `LedDot`, `LedDotLit`, `MeterTrackSm`, `MeterTrackMd`, `PanelSurface`
- Typography: `LabelMonoSm`, `LabelMonoXl`, `LabelBody`, `LabelHeading`
- Any other classes that are required, such that they can be used to apply styles to the Floem components in the same way as they are used in globals.css

### Step 4: Implement styling

Append styling using the floem style classes in `pub fn global_stylesheet(theme: &ThemeColors, dims: &ThemeDimensions) -> Style`.
Translate all CSS rules from `@layer components` in `globals.css` into Floem's fluent `Style` builder syntax:

1. Bind base application canvas background (`app_bg`) and text color (`text_primary`).
2. Map `.class(ClassName, move |s| ...)` blocks for all classes defined in `src/classes.rs`.
3. Handle state variants (`.hover()`, `.active()`, `.focus()`, `.focus_visible()`).
4. Ensure interactive focus states mutate `border_color` rather than using `box_shadow` rings.
5. Ensure grid cells enforce `min_width(0.0)` and `.text_ellipsis()` where appropriate.
