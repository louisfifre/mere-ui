# Glossary

The project's words, each confirmed with the maintainer. Name things this way in code, docs, prompts and conversation.

- **bar**: the strip across the very top of the launcher's window, in place of the system's title bar: the pack icon, the tools, the selected tool's tabs, and the window buttons (minimise, maximise, close). Not the strip, which belongs to the preview page. Rejected: title bar, header, toolbar.
- **tool**: one of the two things the bar switches between, **Home** (the profiles and the dev server) and **World preview**. Rejected: mode, app, page.
- **view**: one way of showing the world: Biomes, Flat, Relief, Temperature, Structures, Section. Rejected: mode, layer.
- **strip**: the row of view tabs at the top of the world preview page; inside the launcher these tabs are drawn in the bar instead. Rejected: tab bar, toolbar.
- **tab**: a button of the strip or of the bar that stays pressed while it is chosen.
- **More**: the last tab of the bar's views when the window lacks room; it opens a list of the views that no longer fit (the greyed ones shown disabled). Not a menu of the bar's other things. Rejected: overflow menu, ellipsis menu.
- **give way**: what the bar does when it lacks room: it drops parts in a set order, worked out from measured widths: in Home the account, then the word MINECRAFT; in the world preview the greyed views, then the other views, then the tools stop being centred and sit left, then 2D and 3D go into More. It is not a breakpoint: nothing is fixed at a number of pixels.
- **panel**: the left side (settings) or the right side (figures, history, list) of the world preview, each as wide as its content wants (or as dragged), beside the stage or, when the window is narrow, stacked under it; the bar's two buttons open and close them. Rejected: column, sidebar.
- **grip**: the invisible zone in the gap between a panel and the stage that a drag resizes the panel from; a 1 px line shows on hover, in the text colour while dragging, and a double-click gives the panel back its natural width. Rejected: handle, splitter.
- **stage**: the centre of the world preview, where the view is drawn (2D map, 3D, or cut); it scales with the window by one continuous rule. Rejected: canvas, viewport, map slot.
- **map**: the 2D view from above, drawn in the stage. Not the 3D view, not the cut.
- **cut**: one vertical slice of the world, drawn by the Section view. Rejected: section (the view's name, not the slice), slice.
- **probe**: the line under the stage that describes the point under the pointer. Rejected: caption, tooltip.
- **band**: one range of heights in the relief's key ("63 to 100"). Rejected: level, range.
- **figures**: the numbers at the top of the right panel (Flat land, Sea, Highest peak…). Rejected: stats, metrics.
- **history**: the small trend lines of the runs before, under the figures.
- **run**: one generation started by Generate preview, with its progress and its stop. Rejected: job, build.
- **out folder**: the folder a run produces and the preview reads (`C:\Games\preview\out` by default). Rejected: data dir.
- **export**: what the Export link writes: the pictures and `export.json`.
- **fold**: a section of a panel that opens and closes (WORLD, SETTINGS…). Rejected: accordion, collapsible.
- **breakpoint**: a window width at which the layout changes form. Kept as few as possible: none in the world preview, at most one in the launcher's Home.
- **shot**: a PNG rendered offscreen, the same frame the window shows. Rejected: screenshot, capture.
- **PLACEHOLDER**: a provisional value shown until the real one exists, always marked as such in the handoff.
