# Naming table of `mere-ui`

This file lists every public name of the crate `crates/mere-ui` (what `lib.rs` re-exports, every `pub fn` of `Ui`, and the
items of `tokens.rs`, one line per family), at the level of atomic design where it belongs: atoms, molecules, organisms,
templates, then the foundations and the utilities that sit under all of them. For each name it gives what it does, the
name it had before the atomic rebuild when it changed, the usual ARIA role, and what the large design systems call the same thing. The ARIA and design
system columns follow the research note `component-naming-research.md` (read 2026-11-23: ARIA APG, HTML, Apple HIG,
Fluent/WinUI, Material, Carbon, Radix, Base UI); for the parts that note does not cover they hold only well-known facts,
and "—" means no system read has the thing.

Rules behind the names: a name is not taken from a physical object ("button", "radio", "checkbox", "switch",
"dropdown"); an item of a list is a "list item" or a "row"; an action (runs something) and an option (selects
something) are named differently. The maintainer left the choice of names to the rebuilding session on 2026-11-23;
the names below are applied in the code, and the Design systems column keeps the usual name beside each.

Status legend:

- confirmed: the name is the maintainer's.
- chosen by the session: a new name, or a rename of a name that broke the rules above, chosen and applied by the
  rebuilding session (its reason is in the row when it is not the rules above).
- keep: the name follows the rules and is unchanged.

Where several names of one family share a row (a function, its `_with` variant, its options and response types), the
Former name cell gives the stem and the others follow it.

## Atoms

| Name | What it does | Former name | Status | ARIA | Design systems |
|---|---|---|---|---|---|
| `selector_mark`, `MarkOpts`, `MARK_SIZE` | The selector alone: square with a check or round with a dot, no label, no press | — | confirmed | `checkbox` / `radio` visual only | Material and Fluent: check mark / radio mark inside a control; Carbon: check box and radio button indicator |
| `Selector::Multiple` | The square selector, several can be selected (the default) | — | confirmed | `menuitemcheckbox` / `checkbox` | Radix: CheckboxItem; Fluent: ToggleMenuFlyoutItem; Atlassian: DropdownItemCheckbox |
| `Selector::Single` | The round selector, one selected among several | — | confirmed | `menuitemradio` / `radio` | Radix: RadioItem; Fluent: RadioMenuFlyoutItem; Atlassian: DropdownItemRadio |
| `INDICATOR_W` | Width of the side bar that marks the selected immediate row | — | chosen by the session | — | — |
| `Rule`, `rule` | The one thin line; `Rule::Section` opens a panel section, `Rule::Separator` parts things inside one | — | chosen by the session (`Rule` and its kinds); `rule` keep | `separator` | Radix, Base UI: Separator; Material: HorizontalDivider; Fluent: MenuFlyoutSeparator; HTML: `hr` |
| `label` | One line of text in a tone | — | keep | none (text); `label` names a control | Material, Carbon: label |
| `text` | A text with a look, cut to its box | — | keep | none | Apple HIG: label (text view) |
| `paragraph` | A text wrapped to its box's width | — | keep | none (`paragraph` in ARIA 1.2) | — |
| `flow` | A lead word then a text wrapped on from it | — | keep | none | — |
| `fill` | A box painted with one colour | — | keep | none (`presentation`) | — |
| `ellipsis`, `ellipsis_start` | A text cut with an ellipsis at its end or at its start | — | keep | none | CSS: `text-overflow: ellipsis` |
| `Icon`, `icon` | A parsed path icon and the call that draws it at a size | — | keep | `img` (decorative: hidden) | Material, Carbon, Fluent: icon |
| `help_mark` | The mark after a label that shows on its row and opens the help | — | keep | `button` with `aria-expanded` | Carbon: toggletip / tooltip trigger; Fluent: InfoLabel |
| `small_icon`, `SMALL_ICON`, `HELP_MARK_W` | A small icon to click beside a text; its size; the mark's hit width | — | keep | `button` | Material: icon button (small) |
| `action`, `action_with`, `ActionOpts`, `ActionLook` | Runs something when pressed: filled, plain, link, running, pick and underlined looks | `button`, `button_with`, `ButtonOpts`, `ButtonLook` | chosen by the session | `button` | Material, Fluent, Carbon, Radix Themes, shadcn: Button; Apple: button |
| `action_icon` | An action with an icon before its word, in the primary or the plain look: the page title's Play and Start | `button_with_icon` | chosen by the session (to validate with the maintainer) | `button` | Material: Button with a leading icon; Carbon: Button with icon; Fluent: Button with icon |
| `split_action`, `SplitOpts` (`list_over`), `Trigger::Split` | A primary action in two parts that always look alike: its word acts, a chevron part (pointing the way the list opens) opens a list of choices, as wide as the button or as the rect `list_over` gives | `split_button`, `SplitButtonOpts` | chosen by the session (to validate with the maintainer) | `button` and `button` with `aria-haspopup` | Material 3: split button; Carbon: combo button; Fluent: SplitButton; Apple: pull-down button |
| `stop_action` | A framed square with a filled square: stops a run | `stop_button` | chosen by the session | `button` | Material: icon button; Apple: button |
| `icon_action`, `IconOpts`, `IconLook` | An action drawn as an icon: tool, caption, close, square, filled, bare looks | `icon_button` | chosen by the session | `button` | Material: IconButton; Fluent: Button with icon only; Carbon: icon-only Button |
| `toggle`, `toggle_with`, `ToggleOpts` | An option that is on or off, with its two words | — | keep ("toggle" names the gesture, not an object) | `switch` | Fluent: ToggleSwitch; Material: Switch; Carbon: Toggle; Apple: toggle; Radix, shadcn: Switch |
| `TOGGLE_W`, `TOGGLE_H` | Size of the toggle's frame | `SWITCH_W`, `SWITCH_H` | chosen by the session (named after the piece they size) | — | — |
| `tab`, `TabOpts`, `TabLook` | One option of a strip or of a row of choices, pressed when current | — | confirmed (a glossary word; it names the option's place in a strip, as ARIA does) | `tab` | Material: Tab; Fluent: Tab; Carbon: Tab; Radix, shadcn: Tabs Trigger; Apple: segmented control |
| `strip_gap` | A gap between two parts of a strip | — | keep | none | — |

## Molecules

| Name | What it does | Former name | Status | ARIA | Design systems |
|---|---|---|---|---|---|
| `selector`, `SelectorOpts` | A selector beside its label; the press is only reported | — | confirmed (kinds) | `checkbox` / `radio` | Material, Fluent, Carbon: Checkbox, RadioButton; Radix: Checkbox, RadioGroup Item |
| `selection_row`, `SelectionRow` | A row of a list that selects: selector, label, optional detail line | — | confirmed | `menuitemcheckbox` / `menuitemradio` / `option` | Radix: CheckboxItem, RadioItem; Fluent: ToggleMenuFlyoutItem, RadioMenuFlyoutItem; Carbon: multiselect option |
| `action_row` | A row of a list that runs something, no selector | — | confirmed | `menuitem` | Radix, Base UI: Item; Fluent: MenuFlyoutItem; Material: DropdownMenuItem; Apple: menu item |
| `immediate_row`, `ImmediateRow` | A row that applies its choice at once; the selected one is lit with a side bar | — | chosen by the session | `option` or `menuitemradio` | Apple: pop-up button item; HTML: `option` |
| `detail_tab` | A strip option with a second, smaller line | — | keep (follows `tab`) | `tab` | — |
| `LIST_ROW`, `LIST_PAD` | Height of a one-line row; space from a list's frame to its first row | — | chosen by the session | — | — |
| `list_row`, `ListRow`, `RowOpts` | A legend row: swatch, name, width, share; pressed when chosen | — | keep | `option` / `listitem` | Carbon: structured list row |
| `list_part`, `PartOpts` | The head of a part of the legend, an action or a plain note | — | keep | `heading` or `button` | — |
| `list_head` | The column heads of the legend | — | keep | `columnheader` | Carbon: table header |
| `trend_row` | A history row: name, value and a plotted line | — | keep | `row` | — |
| `SWATCH_W`, `ROW_GAP`, `WIDTH_MIN`, `SHARE_MIN`, `PLOT_H` | Column and plot measures a panel reads for its natural width | — | keep | — | — |
| `text_field`, `FieldOpts`, `FieldResp`, `FieldLook` | A one-line text input with caret, selection and the input method | — | keep | `textbox` | Material: TextField; Fluent: TextBox; Carbon: TextInput; Apple: text field; shadcn: Input |
| `number`, `number_with`, `NumberSpec`, `NumberOpts` | A number input with min, max, step | — | keep | `spinbutton` | Fluent: NumberBox; Carbon: NumberInput |
| `tool_action`, `tool_action_width`, `TOOL_PAD_L`, `TOOL_PAD_R`, `TOOL_GAP` | A launcher tool: an icon and its name; the width it needs | `tool_button`, `tool_button_width` | chosen by the session | `button` | — |
| `value_row`, `ValueRow`, `ValueResp` | A settings row: label, note and a control in one column grid, with the help mark | — | chosen by the session | `group` or `row` | Fluent: Field; Carbon: form item |
| `value_grid` | The style of the grid the value rows share | — | chosen by the session | — | — |
| `list_line`, `ListLine`, `LineAction`, `LineEnd`, `ListLineResp`, `LINE_NEW_W`, `LINE_ACTION_W`, `LINE_END_W` | A line of a list of versioned things: name and version, then fixed slots (new version, action, icon action) that keep lines of different states aligned | — | chosen by the session (to validate with the maintainer) | `listitem` | Carbon: structured list row; Fluent: DetailsList row |
| `page_head`, `PageHeadResp` | The title line of a page that is not the first: back action, title in its case, what the page offers at its end | — | chosen by the session (to validate with the maintainer) | `heading` with a `button` | Fluent: PageHeader; Material: top app bar |
| `help_label`, `help_anchor` | The label's room that leaves space for the mark; the box the help panel hangs from | — | keep | — | — |
| `help_panel`, `HelpText`, `HelpResp` | The panel that opens over a row with a title, a hint and facts | — | keep | `tooltip` / `dialog` (non-modal) | Carbon: Toggletip; Fluent: Teaching popover; Material: rich tooltip |
| `title`, `TIP_DELAY` | A tooltip's text for a widget; the delay before it shows | — | keep | `tooltip` | Material, Fluent, Carbon, Radix: Tooltip; Apple: help tag |

## Organisms

| Name | What it does | Former name | Status | ARIA | Design systems |
|---|---|---|---|---|---|
| `menu`, `MenuOpts` (`over`), `MenuResp` | A trigger that opens a list of rows above or under it (its chevron pointing that way), as wide as a given rect if one is given; stays open while rows are acted on | — | confirmed | `menu` (list) with a menu button | Radix: DropdownMenu; Base UI: Menu; Apple: pull-down button; Fluent: MenuFlyout; Material: DropdownMenu |
| `Trigger::Line`, `Trigger::Tab` | What opens a menu: a framed line with a chevron, or a strip tab | — | chosen by the session | `button` with `aria-haspopup`, `aria-expanded` | Radix, Base UI: Trigger |
| `group`, `titled_group`, `GroupResp` | Rows stacked with no frame; the titled one has a title in capitals and a separator under it | — | chosen by the session | `group` | Radix, Base UI: Group + Label; HTML: `optgroup`; Apple: section |
| `section`, `ACTIONS_GAP` | A titled section that does not fold, its actions at the right end of its head where a fold has its sign; with no action the section is a `fold` | — | chosen by the session (to validate with the maintainer) | `region` with a heading | Carbon: section heading with actions; Polaris: Card header with actions |
| `select_menu`, `SelectOpts`, `SelectResp` | A line showing the chosen text that opens a list of immediate rows and closes on a pick | — | keep | select-only `combobox` + `listbox` | HTML: select; Apple: pop-up button; Radix, Base UI, MUI: Select; Carbon: Dropdown |
| `tab_menu`, `TabMenuRow`, `TabMenuResp` | The list that hangs from a strip tab ("More"), rows that can be disabled | — | keep (follows `Trigger::Tab`) | `button` + `menu` | Atlassian: dropdown menu with a trigger |
| `fold`, `fold_with`, `FoldOpts` | A titled section whose body opens and closes, the open state the caller's | — | keep | disclosure (`button` + `aria-expanded`) | Carbon: Accordion; Fluent: Accordion; Radix, shadcn: Collapsible, Accordion |
| `scroll`, `ScrollOpts`, `ScrollResp` | A vertical scroll area with the kit's scrollbar | — | keep | `scrollbar` | Fluent: ScrollView; Apple: scroll view |
| `scroll_style`, `arrows_shown`, `GUTTER`, `THUMB_MIN`, `THUMB_W`, `WHEEL_STEP`, `ARROW` | The scroll box's style, which arrows show, and the scrollbar's measures | — | keep | — | — |

## Templates

| Name | What it does | Former name | Status | ARIA | Design systems |
|---|---|---|---|---|---|
| `stage_template`, `StagePlan`, `StageResp` | A stage in the centre and up to two side panels, beside or stacked in a narrow window | — | chosen by the session | `main` + `complementary` landmarks | Material: navigation drawer + supporting pane; Fluent: Drawer |
| `Part` | What the template asks the application to build: a side or the stage | — | chosen by the session | — | — |
| `panel_scroll` | A panel's content as one scrolling whole | — | chosen by the session | `complementary` | — |
| `split_grip`, `GripResp`, `SPLIT_GRIP_W` | The grip in the gap between a panel and the stage that resizes the panel | — | chosen by the session | `separator` (focusable, with a value) | Fluent: Splitter; Material: — |

## Foundations: tokens, theme, text looks, ids, input

| Name | What it does | Former name | Status | ARIA | Design systems |
|---|---|---|---|---|---|
| palette `GREY_*`, `AMBER_*`, `RED_*` (tokens::palette) | Raw colours with no meaning, by step | — | keep | — | Carbon, Material, Radix Colors: colour scale |
| `FRAME` | The one frame, rule and track width | — | keep | — | Fluent: stroke width |
| spacing scale `SPACE_2` ... `SPACE_256` | The only values a space may take | — | keep | — | Carbon: spacing scale; Fluent: spacing tokens |
| spacing roles `INSET_*`, `GAP_*`, `SECTION`, `REGION` | What a space is for: inside a control, between things, between sections | — | keep | — | Material, Fluent: spacing roles |
| sizes `SWATCH`, `ICON`, `CONTROL_COMPACT`, `CONTROL`, `ICON_SQUARE`, `CONTROL_LARGE`, `BAR`, `TIP_WIDTH` | Sizes on a 4 px grid | — | keep | — | Carbon: size tokens |
| fades `FADE_LIT`, `FADE_DIM`, `FADE_DISABLED`, `FADE_TINT` | Opacities over the ground | — | keep | — | Fluent: state layer opacities; Material: state layers |
| `Theme`, `Theme::dark`, `Theme::light` | The roles (ground, text, line, edge...) pointing to the palette, dark and light | — | keep | — | Material, Fluent, Radix: theme / colour roles |
| `Theme::faded`, `Theme::disabled_if`, `Theme::tint`, `mix` | A colour at an opacity over the ground; the disabled and tint colours | — | keep | — | — |
| `TextLook`, `Tone` | A text's size, line, weight, tone and tracking; the tone Text, Muted or Warn | — | keep | — | Material: type scale; Fluent: typography ramp |
| `ADVANCE_EM`, `ASCENT_EM`, `DESCENT_EM`, `XHEIGHT_EM`, `BODY_LINE` | Font metrics and the body line height | — | keep | — | — |
| `Id`, `Id::ROOT`, `Id::child` | A widget's identity from its key path | — | keep | — | — |
| `Input` | The pointer's state and its edges since the last frame | — | keep | — | — |
| `Resp` | What the pointer did to a widget: hovered, held, clicked, pressed, rect | — | keep | — | — |
| `Ui::new`, `begin_frame`, `end_frame`, `event`, `dump` | The frame cycle: build, feed events, lay out and paint, debug dump | — | keep | — | — |
| `Ui::theme`, `set_theme`, `size`, `clock`, `set_clock` | The current theme, the window size, the application's time | — | keep | — | — |
| `Ui::pointer`, `clicks`, `take_wheel`, `rect`, `focused`, `ime_spot`, `menu_open`, `want_redraw`, `set_clipboard`, `wake_in` | Reads of the input and the focus; the input method's spot; the clipboard; when to wake | — | keep | — | — |
| `Ui::cursor`, `want_cursor` | The pointer's shape asked for in the frame | — | keep | — | — |
| `MemoryClipboard` | A clipboard in memory, until the application gives the system's | — | keep | — | — |
| `Slide`, `SLIDE_SECONDS`, `ease` | The page's 220 ms slide and its easing, with reduced motion | — | keep | — | Material, Fluent: motion tokens |

## Utilities

| Name | What it does | Former name | Status | ARIA | Design systems |
|---|---|---|---|---|---|
| `Ui::node` | A container under the open one | — | keep | `generic` | — |
| `Ui::canvas` | A box whose draw commands the application gives | — | keep | `img` or `application` | — |
| `Ui::hover_area`, `Ui::hover_box` | A container that tells where the pointer is over it; the one painted lit while hovered | — | keep | — | — |
| `Ui::hover_action` | A hover box that also takes the press, the widgets inside keeping theirs: a line of a list that opens its page | — | chosen by the session (to validate with the maintainer) | `link` or `button` | Carbon: clickable tile |
| `Ui::overlay` | A framed box painted after the tree, outside every clip | — | keep | `dialog` / `menu` surface | Material, Radix: Portal / Popover content |
| `ellipsize`, `ellipsize_start` | Cut a text to a width with an ellipsis at its end or its start | — | keep | — | — |
| `wrap_lines`, `wrap_lines_after` | Break a text into lines of a width, the second after a lead | — | keep | — | — |
| `text_width`, `cells`, `cells_of` | A text's width in dip; its width in cells | — | keep | — | — |
| `strip_style`, `choice_style`, `details_style` | The styles of the strip, the choices and the details row | — | keep | `tablist` | — |
| `chrome_hit`, `grip`, `GRIP`, `CORNER` | The hit test of a window with its own frame; the sizing edge under a point | — | keep | — | — |
