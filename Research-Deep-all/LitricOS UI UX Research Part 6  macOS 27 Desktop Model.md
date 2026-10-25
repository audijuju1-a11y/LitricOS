# LitricOS UI/UX Research Encyclopedia — Part 6 of N

Chapter 14: macOS 27 desktop model (Batch B5)

| Field | Value |
|---|---|
| Document ID | LITRIC-RES-P06 |
| Revision | 1.0 |
| Target release | macOS 27.0.1 "Golden Gate", build 26A434 (Apple silicon only) |
| Access date | 2026-10-25 |
| Checklist scope | Research C (`MC`): 53 leaf items in 10 groups |
| Official extraction | 82 Apple pages read in full (Mac User Guide 27, Apple Support, macOS 27 release notes, WWDC26 session 289, 13 HIG pages); 1,571 element rows, 1,310 behaviour rows, 445 shortcut mentions |
| Independent extraction | 45 non-Apple pages read in full; 572 claim rows |
| Shortcut register | `macos_shortcuts.csv`: 208 rows (T1) |
| Status (MC leaves) | 2 COMPLETE, 51 PARTIAL |
| Resume point | Part 7 → Batch B6 (MG + G macOS: HIG, Liquid Glass, SF, motion) |

---

## Master Index (updated)

| Ch. | Title | Status |
|---|---|---|
| 0–6 | Baseline, checklist, plan (Part 1) | Published |
| 10 | Windows 11 shell and system UI (WA) | PARTIAL (Part 3) |
| 11 | Windows 11 26H2 Settings (WB) | PARTIAL (Part 2) |
| 12 | Fluent / WinUI design system (WC) | 2 COMPLETE, 11 PARTIAL (Part 4) |
| 13 | macOS 27 System Settings (MD) | 1 COMPLETE, 37 PARTIAL (Part 5) |
| 14 | macOS 27 desktop model (MC) | 2 COMPLETE, 51 PARTIAL (this part) |
| 15 | macOS design system: HIG, Liquid Glass, SF, motion (MG) | Next (Part 7) |
| 16–45 | Remaining chapters | PENDING |

---

## 14.0 Method, Status Rules and Limitations

1. For each of the 53 leaves I ran 1–3 searches restricted to support.apple.com (86 queries, 661 hits) plus 8 gap-filling queries. Topic IDs were normalised to `/<topic>/27/mac/27`.
2. Each page was read in full. A structured extraction recorded UI elements (exact labels), behaviours (trigger → result, with conditions), shortcuts, System Settings paths, explicit macOS 27 changes and API names. It was told to record only what the page states.
3. For changes in macOS 27 and contradictions, I ran 16 searches excluding apple.com (128 hits) and read 45 pages in full. Claims from Dutch, German, French, Polish, Japanese and Korean pages were machine-translated and are marked "(translated)"; the original text is kept in `mc_raw2.json`.
4. Separation (Rule 5): Mac User Guide/Support pages describe OS behaviour. HIG pages are **developer guidance** (labelled `HIG`) and do not show that the OS behaves that way. Release notes are T1 evidence for SDK- and API-level behaviour.
5. Tiers: T1 = Apple; T2 = source repositories or issue trackers; T3 = Wikipedia; T4 = press and blogs; T5 = forums. Beta-era reports (June–August 2026) are flagged because behaviour can change before GA.
6. Limitations: no live Mac. Measurements (corner radii, shadow, animation timing, Dock geometry) are not published by Apple and are deferred to VP-18–VP-22. Apple artwork has not been copied.

### 14.0.1 Headline macOS 27 changes (cross-source)

| ID | Change | Apple evidence | Independent evidence | Confidence |
|---|---|---|---|---|
| MC.H01 | Menu item images are hidden by default. NSMenu hides symbol images, and apps linked on the 27 SDK also hide non-symbol images. `NSMenuItem.preferredImageVisibility` overrides this. In SwiftUI, `labelStyle(.titleAndIcon)` keeps icons. | T1 [macOS 27 release notes](https://developer.apple.com/documentation/macos-release-notes/macos-27-release-notes) | [Daring Fireball](https://daringfireball.net/2026/06/macos_27_golden_gate_removes_the_dumb_icons_from_menu_items), [MacRumors](https://www.macrumors.com/2026/06/11/macos-27-golden-gate-menu-items/), [Blake Crosley](https://blakecrosley.com/ko/blog/menu-images-hidden-macos-27) | High |
| MC.H02 | Open/Save panels: Command-Shift-F opens Recents. Window > Move & Resize and Full Screen Tile work on panels that are not sheets. | T1 [macOS 27 release notes](https://developer.apple.com/documentation/macos-release-notes/macos-27-release-notes) | [BetaProfile](https://betaprofile.dev/macos-27-rc/) | High (confirms Part 1 R-M-001) |
| MC.H03 | Redesigned traffic-light controls with Liquid Glass shading and a press "bounce". Inactive windows show grey buttons. | None found (GAP-B5-01) | [9to5Mac](https://9to5mac.com/2026/08/17/macos-27-golden-gate-beta-6-features-redesigned-traffic-light-window-controls/), [Daring Fireball](https://daringfireball.net/linked/2026/08/18/golden-gate-27-window-controls), [iCulture](https://www.iculture.nl/nieuws/mac-stoplicht-knoppen-liquid-glass-macos-27/), [Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/), [Wikipedia](https://en.wikipedia.org/wiki/MacOS_Golden_Gate) | Medium (beta 6 onward; no T1) |
| MC.H04 | Consistent, less-rounded window corners in all apps, edge-to-edge sidebars, stronger window shadows. | T1 (developer): sidebars extend to window edges, plus corner-concentricity APIs ([WWDC26 session 289](https://developer.apple.com/videos/play/wwdc2026/289/)) | [Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/), [Lets Suite](https://www.lets-suite.com/blog/macos-27-window-management), [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/3/) | Medium (no numbers) |
| MC.H05 | Dock keeps a light-grey indicator for apps still running in the background after quitting, with a hover label and a context-menu item to stop them. | Indirect T1: release notes fix "the Dock … might incorrectly indicate that some applications are running in the background" ([macOS 27 release notes](https://developer.apple.com/documentation/macos-release-notes/macos-27-release-notes)) | [iFun](https://www.ifun.de/macos-27-dock-zeigt-aktive-hintergrund-apps-an-285370/), [iMagazine](https://imagazine.pl/2026/06/10/macos-27-golden-gate-aplikacje-dzialajace-w-tle-dock/), [apple-hacks](https://www.apple-hacks.com/entry/macos-27-golden-gate-background-apps-dock-indicator) | Medium |
| MC.H06 | Menu bar overflow: when the notch or app menus cover status items, a double-arrow control reveals them. | Not in the fetched guide pages | [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/3/), [MacGeneration](https://www.macg.co/macos/2026/06/la-barre-des-menus-de-macos-27-masque-automatiquement-ses-icones-309075), [GitHub jacklandrin/OnlySwitch](https://github.com/jacklandrin/OnlySwitch/issues/218) | Medium |
| MC.H07 | Status items are drawn by one `MenuBarAgent`, and `CGWindowList` no longer exposes them. | — | [GitHub jordanbaird/Ice](https://github.com/jordanbaird/Ice/pull/995) (T2, reverse-engineered) | Low–medium |
| MC.H08 | Screen recording can include system audio ("Include System Audio"). | T1 [Apple Support 102618](https://support.apple.com/en-us/102618) | [Wikipedia](https://en.wikipedia.org/wiki/MacOS_Golden_Gate) | High |
| MC.H09 | iPhone Mirroring windows are resizable (corner drag; Command-9 restores the size). | T1 [Apple Support 120421](https://support.apple.com/en-us/120421) | [9to5Mac](https://9to5mac.com/2026/09/14/macos-27-golden-gate-now-available-here-is-everything-new/) | High |
| MC.H10 | Hot Corners can be suppressed by apps with `.disableScreenCornerInteractions`. | T1 [macOS 27 release notes](https://developer.apple.com/documentation/macos-release-notes/macos-27-release-notes) | — | High (API) |
| MC.H11 | Spotlight hosts Siri AI ("Ask Siri"; Command–Space bar starts a Siri conversation when Siri AI (Beta) is on). | T1 [Mac keyboard shortcuts](https://support.apple.com/en-us/102650), [Spotlight actions](https://support.apple.com/guide/mac-help/mchl4953dfeb/27/mac/27) | [Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/) | High |


## 14.1 MC.01 Menu bar


### MC.01.01 Apple menu

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).


#### What’s in the Apple menu on Mac?

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp1130/27/mac/27) · macOS 27 text present: no

The Apple menu includes options for viewing Mac information, opening settings and the App Store, accessing recent items, managing apps and power, locking the screen, and logging out.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.01.01.e001 | Apple menu | menu | Mac menu bar | Menu containing the listed Mac options. |
| MC.01.01.e002 | About This Mac | menu item | Apple menu | View information about the Mac, including the macOS version, processor or chip type, and memory. |
| MC.01.01.e003 | More Info | button | About This Mac | Opens additional details and options to optimize storage. |
| MC.01.01.e004 | System Settings | menu item | Apple menu | Opens System Settings. |
| MC.01.01.e005 | App Store | menu item | Apple menu | Opens the App Store. |
| MC.01.01.e006 | Open App Store for me | other | App Store description | Link to open the App Store. |
| MC.01.01.e007 | Recent Items | menu item | Apple menu | Shows recent files, apps, and network computers (servers) that have been used. |
| MC.01.01.e008 | Force Quit | menu item | Apple menu | Option for quitting an app that is not responding. |
| MC.01.01.e009 | Sleep | menu item | Apple menu | Dims the screen and reduces power consumption without turning off the Mac. |
| MC.01.01.e010 | Restart | menu item | Apple menu | Turns the Mac off briefly, then turns it on again automatically. |
| MC.01.01.e011 | Shut Down | menu item | Apple menu | Turns the Mac off completely. |
| MC.01.01.e012 | Lock Screen | menu item | Apple menu | Immediately locks the screen. |
| MC.01.01.e013 | Log Out [user name] | menu item | Apple menu | Logs out of the current user account. |
| MC.01.01.e014 | Open recently used items | other | Recent Items description | Link to instructions about opening recently used items. |
| MC.01.01.e015 | Lock the screen of your Mac | other | Lock Screen description | Link to instructions about locking the Mac screen. |
| MC.01.01.e016 | What’s in the menu bar on Mac? | other | See also | Related support page. |
| MC.01.01.e017 | What are those symbols shown in menus on Mac? | other | See also | Related support page. |
| MC.01.01.e018 | Shut down or restart your Mac | other | See also | Related support page. |
| MC.01.01.e019 | Change Users & Groups settings on Mac | other | See also | Related support page. |
| MC.01.01.e020 | If an app freezes or quits unexpectedly on Mac | other | See also | Related support page. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.01.01.b001 | Choose About This Mac from the Apple menu. | View information about the Mac, such as its macOS version, processor or chip type, and memory. | — |
| MC.01.01.b002 | Click More Info. | View additional details and options to optimize storage. | From About This Mac. |
| MC.01.01.b003 | Choose System Settings from the Apple menu. | Open System Settings to customize Mac settings and update macOS. | — |
| MC.01.01.b004 | Choose App Store from the Apple menu. | Visit the App Store to find apps and updates for apps, printers, and more. | — |
| MC.01.01.b005 | Use the Mac and access Recent Items. | View a list of recently used files, apps, and network computers (servers); their names are added to the list for quick access. | As items are used, their names are added. |
| MC.01.01.b006 | Choose Force Quit. | Force an app to quit. | Use when an app is not responding. Unsaved changes may be lost. |
| MC.01.01.b007 | Choose Sleep. | Dim the screen and reduce power consumption without turning off the Mac. | — |
| MC.01.01.b008 | Choose Restart. | Turn the Mac off briefly, then have it turn on again automatically. | — |
| MC.01.01.b009 | Choose Shut Down. | Turn the Mac off completely. | — |
| MC.01.01.b010 | Choose Lock Screen. | Immediately lock the screen to help keep the Mac and information secure. | When needed. |
| MC.01.01.b011 | Choose Log Out [user name]. | Log out of the current user account without shutting down the Mac. | Useful when sharing the Mac with other users who have their own accounts. |

### MC.01.02 App menu & standard menus

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).


#### What’s in the menu bar on Mac?

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp1446/27/mac/27) · macOS 27 text present: no

The menu bar runs along the top of the screen. Its menus and icons let you choose commands, perform tasks, and check your Mac’s status.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.01.02.e001 | menu bar | view | Along the top of the screen | Contains menus and icons used to choose commands, perform tasks, and check status. |
| MC.01.02.e002 | Apple menu | menu | At the left end of the menu bar | Can be used to open System Settings, lock the screen, or shut down the Mac. |
| MC.01.02.e003 | System Settings | other | Opened from the Apple menu | The article names it as an Apple menu action. |
| MC.01.02.e004 | App menus | menu | Next to the Apple menu | The name of the app being used appears in bold, followed by other menus. |
| MC.01.02.e005 | File | menu | Next to the app name in the menu bar | An example of a standard app menu; often contains the Open command. |
| MC.01.02.e006 | Edit | menu | Next to the app name in the menu bar | An example of a standard app menu. |
| MC.01.02.e007 | Format | menu | Next to the app name in the menu bar | An example of a standard app menu. |
| MC.01.02.e008 | Window | menu | Next to the app name in the menu bar | An example of a standard app menu. |
| MC.01.02.e009 | Help | menu | In each app’s menu bar | Each app has a Help menu for information about using the app. |
| MC.01.02.e010 | Open | menu item | Often in the File menu | An example of a command available in many apps. |
| MC.01.02.e011 | status menus | menu | Toward the right end of the menu bar | Items, typically represented by icons, that show Mac status or let you customize features. |
| MC.01.02.e012 | battery charge | indicator | In a status menu | Given as an example of Mac status that can be checked. |
| MC.01.02.e013 | keyboard brightness | setting | Associated with a status menu | Given as an example of a feature that can be customized. |
| MC.01.02.e014 | Spotlight icon | icon | In the menu bar | The source text associates it with “Search with Spotlight.” |
| MC.01.02.e015 | Control Center | menu | In the menu bar | The article says to click Stage Manager, Screen Mirroring, Focus, and more. |
| MC.01.02.e016 | Stage Manager | other | In Control Center | Named as an item to click. |
| MC.01.02.e017 | Screen Mirroring | other | In Control Center | Named as an item to click. |
| MC.01.02.e018 | Focus | other | In Control Center | Named as an item to click. |
| MC.01.02.e019 | Privacy indicators | indicator | To the right of Control Center | Can be dots or arrows. The source text ends after mentioning an orange dot. |
| MC.01.02.e020 | Notification Center | view | Opened from the date and time at the right end of the menu bar | Shows widgets and notifications that were missed. |
| MC.01.02.e021 | date and time | control | At the right end of the menu bar | Clicking it opens Notification Center. |
| MC.01.02.e022 | widgets | view | In Notification Center | Can be viewed in Notification Center. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.01.02.b001 | Set the option to automatically hide the menu bar | The menu bar is shown only when the pointer is moved to the top of the screen. | The automatic-hide option must be set. |
| MC.01.02.b002 | Use the Apple menu | Open System Settings, lock the screen, or shut down the Mac. | — |
| MC.01.02.b003 | Choose a command from an app menu | The command is performed. | Each menu contains commands; many are available in most apps. |
| MC.01.02.b004 | Click a status menu icon | More details or options are shown. | — |
| MC.01.02.b005 | Press and hold Command while dragging a status menu icon | Rearrange the status menu. | The icon is in the menu bar. |
| MC.01.02.b006 | Press and hold Command while dragging a status menu icon out of the menu bar | Quickly remove the status menu. | — |
| MC.01.02.b007 | Click Stage Manager, Screen Mirroring, Focus, or another item in Control Center | The named item can be accessed from Control Center. | — |
| MC.01.02.b008 | Click the date and time | Open Notification Center, where widgets and missed notifications can be viewed. | The date and time are at the right end of the menu bar. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.01.02.k001 | Command | Hold while dragging a status menu icon to rearrange it or drag it out of the menu bar to remove it. | Menu bar status menus |

System Settings paths referenced: Change Menu Bar settings

#### The menu bar

T1 · HIG (developer guidance) · [Apple HIG](https://developer.apple.com/design/human-interface-guidelines/the-menu-bar) · macOS 27 text present: no

Guidance for organizing and labeling macOS menu bar menus, their standard commands, dynamic items, and menu bar extras. No macOS 27 or Golden Gate changes are stated.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.01.02.e023 | Apple menu | menu | Leading side of the macOS menu bar | Always the first item. Contains system-defined items and cannot be modified or removed. |
| MC.01.02.e024 | YourAppName | menu | Menu bar, after the Apple menu | The app menu; use a short app name. The active app name appears in bold. |
| MC.01.02.e025 | File | menu | Menu bar, after the app menu | Contains commands for managing supported files or documents. It can be renamed or omitted if the app handles no file types. |
| MC.01.02.e026 | Edit | menu | Menu bar | Contains editing and Clipboard commands. |
| MC.01.02.e027 | Format | menu | Menu bar | Contains text-formatting commands; it can be omitted if the app does not support formatted text editing. |
| MC.01.02.e028 | View | menu | Menu bar | Contains commands for customizing the appearance of the app’s windows. |
| MC.01.02.e029 | App-specific menus | menu | Between View and Window | Menus for custom app commands. |
| MC.01.02.e030 | Window | menu | Menu bar | Contains commands for navigating, organizing, and managing app windows. |
| MC.01.02.e031 | Help | menu | Trailing end of the menu bar | Provides access to app help documentation. macOS adds a search field at the top when help uses the Help Book format. |
| MC.01.02.e032 | Menu bar extras | icon | Trailing side of the macOS menu bar | App-specific functionality shown as an icon, including when the app is not frontmost. The system may hide extras to make room for app menus or reduce crowding. |
| MC.01.02.e033 | About YourAppName | menu item | YourAppName menu | Opens the About window with copyright and version information. Prefer an app name of 16 characters or fewer and omit a version number. |
| MC.01.02.e034 | Settings… | menu item | YourAppName menu | Opens the app’s settings window on macOS; use it only for app-level settings. |
| MC.01.02.e035 | Services | menu item | YourAppName menu | Displays a submenu of relevant services from the system and other apps. |
| MC.01.02.e036 | Hide YourAppName | menu item | YourAppName menu | Hides the app and all its windows, then activates the most recently used app. |
| MC.01.02.e037 | Hide Others | menu item | YourAppName menu | Hides all other open apps and their windows. |
| MC.01.02.e038 | Show All | menu item | YourAppName menu | Shows all other open apps and their windows behind this app’s windows. |
| MC.01.02.e039 | Quit YourAppName | menu item | YourAppName menu | Quits the app. |
| MC.01.02.e040 | New Item | menu item | File menu | Creates a new document, file, or window; replace Item with the type of item the app creates. |
| MC.01.02.e041 | Open | menu item | File menu | Opens a selected item or presents an interface for selecting an item. Add an ellipsis when a separate interface is required. |
| MC.01.02.e042 | Open Recent | menu item | File menu | Submenu of recently opened documents and files, typically including Clear Menu. |
| MC.01.02.e043 | Close | menu item | File menu | Closes the current window or document. In a tab-based window, Close Tab replaces it. |
| MC.01.02.e044 | Close Tab | menu item | File menu | Closes the current tab in a tab-based window. |
| MC.01.02.e045 | Close File | menu item | File menu | Closes the current file and all its associated windows. |
| MC.01.02.e046 | Save | menu item | File menu | Saves the current document or file. |
| MC.01.02.e047 | Save All | menu item | File menu | Saves all open documents. |
| MC.01.02.e048 | Duplicate | menu item | File menu | Duplicates the current document and leaves both documents open. |
| MC.01.02.e049 | Rename… | menu item | File menu | Lets people change the current document’s name. |
| MC.01.02.e050 | Move To… | menu item | File menu | Prompts people to choose a new document location. |
| MC.01.02.e051 | Export As… | menu item | File menu | Prompts for a name, output location, and format; the exported file does not open and the current document remains open. |
| MC.01.02.e052 | Revert To | menu item | File menu | When autosaving is on, provides recent document versions and access to the version browser. |
| MC.01.02.e053 | Page Setup… | menu item | File menu | Opens a panel for document-specific printing parameters such as paper size and orientation. |
| MC.01.02.e054 | Print… | menu item | File menu | Opens the standard Print panel. |
| MC.01.02.e055 | Undo | menu item | Edit menu | Reverses the previous user operation; the title can clarify what is undone. |
| MC.01.02.e056 | Redo | menu item | Edit menu | Reverses the previous Undo operation; the title can clarify what is redone. |
| MC.01.02.e057 | Cut | menu item | Edit menu | Removes selected data and stores it on the Clipboard. |
| MC.01.02.e058 | Copy | menu item | Edit menu | Duplicates selected data and stores it on the Clipboard. |
| MC.01.02.e059 | Paste | menu item | Edit menu | Inserts Clipboard contents at the insertion point without changing the Clipboard. |
| MC.01.02.e060 | Paste and Match Style | menu item | Edit menu | Pastes text using the style of the surrounding text. |
| MC.01.02.e061 | Delete | menu item | Edit menu | Removes selected data without placing it on the Clipboard. |
| MC.01.02.e062 | Select All | menu item | Edit menu | Highlights all selectable content in the current document or text container. |
| MC.01.02.e063 | Find | menu item | Edit menu | Submenu for search operations, including Find, Find and Replace, Find Next, Find Previous, Use Selection for Find, and Jump to Selection. |
| MC.01.02.e064 | Spelling and Grammar | menu item | Edit menu | Submenu for spelling and grammar tools. |
| MC.01.02.e065 | Substitutions | menu item | Edit menu | Submenu for automatic typing substitutions. |
| MC.01.02.e066 | Transformations | menu item | Edit menu | Submenu for transforming selected text. |
| MC.01.02.e067 | Speech | menu item | Edit menu | Submenu with Start Speaking and Stop Speaking. |
| MC.01.02.e068 | Start Dictation | menu item | Bottom of the Edit menu | Opens dictation and inserts converted speech at the current insertion point; the system adds this item automatically. |
| MC.01.02.e069 | Emoji & Symbols | menu item | Bottom of the Edit menu | Opens Character Viewer; the system adds this item automatically. |
| MC.01.02.e070 | Font | menu item | Format menu | Submenu for font attributes, including Show Fonts, Bold, Italic, Underline, Bigger, Smaller, Show Colors, Copy Style, and Paste Style. |
| MC.01.02.e071 | Text | menu item | Format menu | Submenu for text attributes, including alignment, writing direction, and ruler commands. |
| MC.01.02.e072 | Show/Hide Tab Bar | menu item | View menu | Toggles the tab bar in a tab-based window. |
| MC.01.02.e073 | Show All Tabs/Exit Tab Overview | menu item | View menu | Enters or exits the overview of open tabs in a tab-based window. |
| MC.01.02.e074 | Show/Hide Toolbar | menu item | View menu | Toggles the toolbar’s visibility. |
| MC.01.02.e075 | Customize Toolbar | menu item | View menu | Opens a view for customizing toolbar items. |
| MC.01.02.e076 | Show/Hide Sidebar | menu item | View menu | Toggles the sidebar’s visibility. |
| MC.01.02.e077 | Enter/Exit Full Screen | menu item | View menu; in Window only when the app has no View menu | Enters or exits the full-screen experience. |
| MC.01.02.e078 | Minimize | menu item | Window menu | Minimizes the active window to the Dock. |
| MC.01.02.e079 | Zoom | menu item | Window menu | Toggles between a predefined size appropriate to the window’s content and the size people set. |
| MC.01.02.e080 | Show Previous Tab | menu item | Window menu | Shows the tab before the current tab. |
| MC.01.02.e081 | Show Next Tab | menu item | Window menu | Shows the tab after the current tab. |
| MC.01.02.e082 | Move Tab to New Window | menu item | Window menu | Opens the current tab in a new window. |
| MC.01.02.e083 | Merge All Windows | menu item | Window menu | Combines all open windows into one tabbed window. |
| MC.01.02.e084 | Bring All to Front | menu item | Window menu | Brings all app windows forward while maintaining their onscreen locations, sizes, and layering order. |
| MC.01.02.e085 | Send YourAppName Feedback to Apple | menu item | Help menu | Opens Feedback Assistant. |
| MC.01.02.e086 | YourAppName Help | menu item | Help menu | Opens Help Book content in the built-in Help Viewer. |
| MC.01.02.e087 | Print panel | panel | Opened by File > Print… | Supports printing, faxing, or saving as a PDF. |
| MC.01.02.e088 | About window | window | Opened by About YourAppName | Includes app copyright and version information. |
| MC.01.02.e089 | Help Viewer | window | Opened by YourAppName Help when documentation uses Help Book format | Displays the app’s help documentation. |

| ID | Context | Guideline | Conditions |
|---|---|---|---|
| MC.01.02.b009 | When arranging top-level menus | Use the order YourAppName, File, Edit, Format, View, app-specific menus, Window, Help; macOS places the Apple menu at the leading side and menu bar extras at the trailing side. | Format may be excluded when formatted text editing is unsupported; File may be renamed or omitted if the app handles no file types. |
| MC.01.02.b010 | When providing standard menus and menu items | Support system-defined menus and their familiar ordering; the system implements many standard commands, such as Edit > Copy for selected text in a standard text field. | As applicable to the app’s features. |
| MC.01.02.b011 | When a menu item is unavailable in the current context | Keep it visible and disable its action rather than hiding it. | Applies to menu bar items. |
| MC.01.02.b012 | When representing common actions in menu items | Use familiar icons, matching the system’s icons for actions such as Copy, Share, and Delete. | Where icons are used. |
| MC.01.02.b013 | When including standard menu items | Support their established keyboard shortcuts; define custom shortcuts only when necessary. | For standard items such as Copy, Cut, Paste, Save, and Print. |
| MC.01.02.b014 | When naming menu titles | Prefer short, one-word titles; if a title has more than one word, use title-style capitalization. | Short titles help with spacing and scanning. |
| MC.01.02.b015 | When configuring the app menu | Use it for app-wide actions rather than task-, document-, or window-specific actions; put About first and separate it from the next group. | Custom app-configuration items follow Settings in the same group. |
| MC.01.02.b016 | When naming About and app visibility or quit items | Use the same short app name for About, Hide, and Quit; keep the About name to 16 characters or fewer and do not include a version number. | As applicable to the app menu. |
| MC.01.02.b017 | When implementing Save and document actions | Autosave periodically; prompt for a name and location for a new document; use a format pop-up in the Save sheet when saving in multiple formats. | For apps that support document saving. |
| MC.01.02.b018 | When deciding where to put Find commands | Use the Edit menu for searching the current document or text container; consider File when searching files or other objects. | Depends on what the app’s Find commands search. |
| MC.01.02.b019 | When the app supports only some standard view functions | Still provide a View menu with the supported functions, such as Enter/Exit Full Screen. | Applies even if the app has no tab bar, toolbar, or sidebar. |
| MC.01.02.b020 | When a view is hidden or visible | Make each show/hide item’s title describe the action available in the current state, such as Show Toolbar when hidden and Hide Toolbar when visible. | For corresponding views in the View menu. |
| MC.01.02.b021 | When adding app-specific menus | Put custom commands in the menu bar, reflect the app’s hierarchy where possible, and order menus from more general or commonly used toward more specialized. | App-specific menus appear between View and Window. |
| MC.01.02.b022 | When providing a Window menu | Provide it even for a single-window app, and include Minimize and Zoom. | Supports people using Full Keyboard Access. |
| MC.01.02.b023 | When adding window-management commands | Use Window for navigating and managing windows; use View for appearance customization and File > Close for closing windows. | Do not use Zoom to enter or exit full screen. |
| MC.01.02.b024 | When deciding whether to include Enter/Exit Full Screen in Window | Include it there only if the app has no View menu, and continue to provide separate Minimize and Zoom items. | Otherwise, place the full-screen item in View. |
| MC.01.02.b025 | When listing open app-specific windows | List them alphabetically and do not list panels or other modal views. | In the Window menu. |
| MC.01.02.b026 | When people press Option while choosing certain menu items | Close changes to Close All; Close Tab changes to Close Other Tabs; Duplicate changes to Save As; Minimize changes to Minimize All; Zoom changes to Zoom All; Bring All to Front changes to Arrange in Front; Quit YourAppName changes to Quit and Keep Windows. | These are dynamic menu-item alternatives. |
| MC.01.02.b027 | When using dynamic menu items | Do not make a dynamic item the only way to accomplish a task; use them primarily in menu bar menus and require only one modifier key. | Dynamic items are hidden by default; macOS sizes a menu to fit its widest item, including dynamic items. |
| MC.01.02.b028 | When menu bar space is constrained | The system prioritizes menus and essential menu bar extras and may reduce spacing or truncate menu titles. | On macOS. |
| MC.01.02.b029 | When the app enters full-screen mode | The menu bar typically hides until people move the pointer to the top of the screen. | On macOS. |
| MC.01.02.b030 | When people click a menu bar extra | Display a menu rather than a popover. | Avoid a menu only when the functionality is too complex for one. |
| MC.01.02.b031 | When deciding whether a menu bar extra appears | Let people choose whether to put it in the menu bar, typically through an app setting; consider offering the choice during setup. | Do not rely on its presence because the system may hide it and its position is unpredictable. |
| MC.01.02.b032 | When designing a menu bar extra icon | Consider an SF Symbol or a custom symbol; the system can adapt black areas for light and dark menu bars and selected states. | The menu bar height is 24 pt. |
| MC.01.02.b033 | When exposing app-specific functionality | Consider additional access methods such as a Dock menu, available when people Control-click the app’s Dock icon. | Menu bar extras can be hidden or unused. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.01.02.k002 | Option | Changes Quit YourAppName to Quit and Keep Windows. | YourAppName menu. |
| MC.01.02.k003 | Option | Changes Close to Close All; Close Tab to Close Other Tabs; Duplicate to Save As. | File menu. |
| MC.01.02.k004 | Option | Changes Minimize to Minimize All; Zoom to Zoom All; Bring All to Front to Arrange in Front. | Window menu. |
| MC.01.02.k005 | Control | Opens the Dock menu. | Control-click the app’s Dock icon. |
| MC.01.02.k006 | Standard keyboard shortcuts | Support the established shortcuts for standard menu items such as Copy, Cut, Paste, Save, and Print. | The page does not specify key combinations. |

API names: `isAlternate`, `NSHelpManager`, `CommandMenu`, `Adding menus and shortcuts to the menu bar and user interface`, `NSStatusBar`, `Menu`

### MC.01.03 Menu item image visibility (27 change)

Status: **COMPLETE**. SDK-dependent menu-image rules are documented in the macOS 27 release notes (T1) and corroborated by 5 independent sources; API NSMenuItem.preferredImageVisibility.

Also covered by: [What’s in the menu bar on Mac?](https://support.apple.com/guide/mac-help/mchlp1446/27/mac/27)


#### Customize the menu bar on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchl4af84660/27/mac/27) · macOS 27 text present: yes

Add icons for settings and controls to the menu bar from System Settings or Control Center, or remove icons you added.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.01.03.e001 | Apple menu | menu | At the top of the screen | Menu to choose when opening Menu Bar settings. |
| MC.01.03.e002 | Open Menu Bar settings for me | menu item | After choosing Apple menu | Link to open Menu Bar settings. |
| MC.01.03.e003 | System Settings | app | Not specified | Used to add icons for settings and controls to the menu bar. |
| MC.01.03.e004 | Menu Bar settings | view | System Settings | Settings where icons can be added or removed. |
| MC.01.03.e005 | menu bar | view | At the top of the screen | Area where icons can be added or removed. |
| MC.01.03.e006 | Control Center | view | Accessed from the menu bar | Can be used to add icons to the menu bar. |
| MC.01.03.e007 | Edit Controls | button | Control Center | Opens controls for editing and adding items to the menu bar. |
| MC.01.03.e008 | Controls Gallery | view | After clicking Edit Controls | Source of items that can be dragged to the menu bar. |
| MC.01.03.e009 | Siri | icon | Menu bar | Example of a setting or control that can be added from System Settings. |
| MC.01.03.e010 | Weather | icon | Menu bar | Example of a setting or control that can be added from System Settings. |
| MC.01.03.e011 | Time Machine | icon | Menu bar | Example of a setting or control that can be added from System Settings. |
| MC.01.03.e012 | Quick Note | icon | Menu bar | Example of a setting or control that can be added using Control Center. |
| MC.01.03.e013 | Put Display to Sleep | icon | Menu bar | Example of a setting or control that can be added using Control Center. |
| MC.01.03.e014 | Color filters | icon | Menu bar | Example of a setting or control that can be added using Control Center. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.01.03.b001 | Choose Apple menu, then choose “Open Menu Bar settings for me.” | Open Menu Bar settings. | For adding or removing an icon added from System Settings. |
| MC.01.03.b002 | In System Settings, add an icon for a setting or control. | The icon is added to the menu bar. | Examples given are Siri, Weather, and Time Machine. |
| MC.01.03.b003 | Click the menu bar, click “Edit Controls,” then drag an item from the Controls Gallery or Control Center to the menu bar. | The item is added to the menu bar. | Examples given are Quick Note, Put Display to Sleep, and Color filters. |
| MC.01.03.b004 | In Menu Bar settings, remove an icon you added from System Settings. | The icon is removed from the menu bar. | Applies to an icon added from System Settings. |
| MC.01.03.b005 | Click the menu bar, click “Edit Controls,” then drag an item off the menu bar. | The item is removed from the menu bar. | Applies to an icon added from Control Center. |

System Settings paths referenced: System Settings > Menu Bar settings

#### Change Menu Bar settings on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlad96d366/27/mac/27) · macOS 27 text present: no

The page describes settings for choosing what appears in the Mac menu bar, how the menu bar is displayed, and how certain status items behave. It also mentions customizing Control Center and resetting its settings.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.01.03.e015 | Apple menu | menu | Referenced in the introductory instructions | The page says to choose this menu to change these settings; the remainder of the instruction is not included. |
| MC.01.03.e016 | Menu Bar settings | panel | Settings described on the page | Settings for menu bar appearance and included items. |
| MC.01.03.e017 | General options | view | Menu Bar settings | Section containing menu bar display and recent-items options. |
| MC.01.03.e018 | Automatically hide and show the menu bar | setting | General options | Choose when the menu bar is hidden and shown. |
| MC.01.03.e019 | Always | menu item | Automatically hide and show the menu bar options | Always hide and show the menu bar. |
| MC.01.03.e020 | On Desktop Only | menu item | Automatically hide and show the menu bar options | Hide and show the menu bar when not using an app in full screen. |
| MC.01.03.e021 | In Full Screen Only | menu item | Automatically hide and show the menu bar options | Hide and show the menu bar when using an app in full screen. |
| MC.01.03.e022 | Never | menu item | Automatically hide and show the menu bar options | Never hide the menu bar. |
| MC.01.03.e023 | Show menu bar background | setting | General options | Choose whether to show wallpaper behind the menu bar or show the menu bar background. |
| MC.01.03.e024 | Recent documents, applications, and servers | setting | General options | Choose how many items to show in the Recent Items menu command. |
| MC.01.03.e025 | Recent Items | menu item | Apple menu | Menu command listing recently used items. |
| MC.01.03.e026 | Open Recent | menu item | App menus, such as the File menu in TextEdit | Menu option whose recent-item count is also determined by the Recent documents, applications, and servers setting. |
| MC.01.03.e027 | File | menu | TextEdit | Example app menu containing the Open Recent option. |
| MC.01.03.e028 | Menu Bar Controls | view | Menu Bar settings | Section listing items that can appear in the menu bar; the page notes that more items are available in Control Center. |
| MC.01.03.e029 | Clock | setting | Menu Bar Controls | The clock is always shown in the menu bar. |
| MC.01.03.e030 | Clock Options | button | Beside Clock in Menu Bar Controls | Opens options for how the date and time are displayed in the menu bar clock. |
| MC.01.03.e031 | Siri | setting | Menu Bar Controls | Select to show Siri in the menu bar. |
| MC.01.03.e032 | Spotlight | setting | Menu Bar Controls | Select to show Spotlight in the menu bar. |
| MC.01.03.e033 | Wi-Fi | setting | Menu Bar Controls | Select to show Wi-Fi in the menu bar. |
| MC.01.03.e034 | Bluetooth® | setting | Menu Bar Controls | Select to show Bluetooth in the menu bar. |
| MC.01.03.e035 | Battery | setting | Menu Bar Controls | On a Mac laptop, select to show Battery status in the menu bar. |
| MC.01.03.e036 | Battery Options | button | Beside Battery in Menu Bar Controls | Opens options for showing the battery charge percentage and when to show the energy mode. |
| MC.01.03.e037 | Focus | setting | Menu Bar Controls | Select to show Focus in the menu bar; its pop-up menu provides display options. |
| MC.01.03.e038 | Screen Mirroring | setting | Menu Bar Controls | Select to show Screen Mirroring in the menu bar; its pop-up menu provides display options. |
| MC.01.03.e039 | Display | setting | Menu Bar Controls | Select to show Display in the menu bar; its pop-up menu provides display options. |
| MC.01.03.e040 | Sound | setting | Menu Bar Controls | Select to show Sound in the menu bar; its pop-up menu provides display options. |
| MC.01.03.e041 | Now Playing | setting | Menu Bar Controls | Select to show Now Playing in the menu bar; its pop-up menu provides display options. |
| MC.01.03.e042 | Show When Active | menu item | Pop-up menu next to Focus, Screen Mirroring, Display, Sound, and Now Playing | Show the status icon in the menu bar only when the item is active. |
| MC.01.03.e043 | Always Show | menu item | Pop-up menu next to Focus, Screen Mirroring, Display, Sound, and Now Playing | Show the status icon in the menu bar at all times. |
| MC.01.03.e044 | Fast User Switching | setting | Menu Bar Controls | Select to show Fast User Switching in the menu bar; its pop-up menu provides display choices. |
| MC.01.03.e045 | Full Name | menu item | Pop-up menu next to Fast User Switching | Show the full name of the user in the menu bar. |
| MC.01.03.e046 | Account Name | menu item | Pop-up menu next to Fast User Switching | Show the name of the user account in the menu bar. |
| MC.01.03.e047 | Icon | menu item | Pop-up menu next to Fast User Switching | Show the Fast User Switching icon. |
| MC.01.03.e048 | Time Machine | setting | Menu Bar Controls | Select to show Time Machine in the menu bar. |
| MC.01.03.e049 | VPN | setting | Menu Bar Controls | Select to show VPN in the menu bar. |
| MC.01.03.e050 | Weather | setting | Menu Bar Controls | Select to show Weather in the menu bar. |
| MC.01.03.e051 | Allow in the Menu Bar | view | Menu Bar settings | Section for menu bar items supplied by applications. |
| MC.01.03.e052 | [Application name] | setting | Allow in the Menu Bar | Select to show an application's menu bar item. |
| MC.01.03.e053 | Reset Option | view | Menu Bar settings | Section containing the Control Center reset option. |
| MC.01.03.e054 | Reset Control Center | button | Reset Option | Returns Control Center to its original settings. |
| MC.01.03.e055 | Control Center | view | Referenced in the introduction and Menu Bar Controls descriptions | More items are available here; items can also be dragged directly to the menu bar when customizing it. |
| MC.01.03.e056 | Customize the menu bar | other | Introductory See link | Related topic referenced by the page. |
| MC.01.03.e057 | Use Control Center | other | Introductory See link | Related topic referenced by the page. |
| MC.01.03.e058 | Change the menu bar clock format | other | Clock description | Related topic referenced by the page. |
| MC.01.03.e059 | What’s in the menu bar on Mac? | other | See also | Related topic referenced by the page. |
| MC.01.03.e060 | Turn a Focus on or off on Mac | other | See also | Related topic referenced by the page. |
| MC.01.03.e061 | Change Date & Time settings on Mac | other | See also | Related topic referenced by the page. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.01.03.b006 | Choose Always for Automatically hide and show the menu bar | The menu bar is always hidden and shown. | General options. |
| MC.01.03.b007 | Choose On Desktop Only for Automatically hide and show the menu bar | The menu bar is hidden and shown when not using an app in full screen. | General options. |
| MC.01.03.b008 | Choose In Full Screen Only for Automatically hide and show the menu bar | The menu bar is hidden and shown when using an app in full screen. | General options. |
| MC.01.03.b009 | Choose Never for Automatically hide and show the menu bar | The menu bar is never hidden. | General options. |
| MC.01.03.b010 | Move the pointer to the top of the screen | The menu bar appears. | When the menu bar is hidden. |
| MC.01.03.b011 | Change Show menu bar background | Choose whether wallpaper appears behind the menu bar or the menu bar background is shown. | General options. |
| MC.01.03.b012 | Change Recent documents, applications, and servers | The chosen number of items is shown in Recent Items and also determines how many recent items appear in app menus, such as Open Recent in TextEdit’s File menu. | General options. |
| MC.01.03.b013 | Click Clock Options | Choose how the date and time are displayed in the menu bar clock. | The clock is always shown. |
| MC.01.03.b014 | Select Battery | Show Battery status in the menu bar. | Mac laptop. |
| MC.01.03.b015 | Click Battery Options | Choose whether to show the battery charge percentage and when to show the energy mode. | Battery settings; the battery percentage is always shown when Battery is in Control Center. |
| MC.01.03.b016 | Select Siri, Spotlight, Wi-Fi, or Bluetooth® | Show the selected item in the menu bar. | Menu Bar Controls. |
| MC.01.03.b017 | Select Focus, Screen Mirroring, Display, Sound, or Now Playing | Show the selected item in the menu bar. | Menu Bar Controls. |
| MC.01.03.b018 | Choose Show When Active from the pop-up menu for Focus, Screen Mirroring, Display, Sound, or Now Playing | Show that item's status icon only while the item is active. | The item is selected to appear in the menu bar. |
| MC.01.03.b019 | Choose Always Show from the pop-up menu for Focus, Screen Mirroring, Display, Sound, or Now Playing | Show that item's status icon at all times. | The item is selected to appear in the menu bar. |
| MC.01.03.b020 | Select Fast User Switching | Show Fast User Switching in the menu bar. | Menu Bar Controls. |
| MC.01.03.b021 | Choose Full Name, Account Name, or Icon from the Fast User Switching pop-up menu | Show the full user name, account name, or Fast User Switching icon, respectively. | Fast User Switching is selected to appear in the menu bar. |
| MC.01.03.b022 | Select Time Machine, VPN, or Weather | Show the selected item in the menu bar. | Menu Bar Controls. |
| MC.01.03.b023 | Select an application under Allow in the Menu Bar | Show that application's item in the menu bar. | The application provides a menu bar item. |
| MC.01.03.b024 | Customize Control Center and drag an item directly to the menu bar | Add the item to the menu bar. | The item is available in Control Center. |
| MC.01.03.b025 | Click Reset Control Center | Return Control Center to its original settings. | Reset Option. |

System Settings paths referenced: Menu Bar settings

#### Modernize your AppKit app

T1 · Developer session · [WWDC26 session 289](https://developer.apple.com/videos/play/wwdc2026/289/) · macOS 27 text present: yes

The WWDC26 session describes modern AppKit input handling, keyboard navigation, graceful termination and state restoration, and macOS 27 design updates including Liquid Glass refinements and corner concentricity.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.01.03.e062 | NSStatusItem | other | Menu bar | A status item can act as a menu, trigger an action, or present custom UI such as a window. |
| MC.01.03.e063 | Sidebar | view | Window edge | In macOS 27, sidebars extend to the window's edges, with content continuing behind them. |
| MC.01.03.e064 | Bordered toolbar items | control | Toolbar over a sidebar | These items adopt Liquid Glass in macOS 27. |
| MC.01.03.e065 | Liquid Glass effect | other | Interactive controls and glass containers | A new effect subtly bounces the glass when clicked. The session recommends using it selectively for controls, buttons, and containers of interactive controls. |
| MC.01.03.e066 | Concentric corners | other | Views near a container's corners | A view's corner radius can adapt to the shape of its container, matching it more closely as the view approaches the container's corner. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.01.03.b026 | A view or control receives user input | Gesture recognizers provide modern event handling across AppKit and other Mac UI frameworks, instead of relying on mouseDown overrides and tracking loops. | Use view APIs, control events, or gesture recognizers according to the interaction. |
| MC.01.03.b027 | Selection changes in a table or collection view | Observe selected state or use delegate callbacks to respond to the selection change. | Prefer these APIs over tracking selection in a mouseDown override. |
| MC.01.03.b028 | A context menu is requested | The view can show a shared default menu, a responder-specific menu, or a menu created for the event. | Use defaultMenu, menu, or menuForEvent as appropriate. |
| MC.01.03.b029 | A row or item is dragged from a collection-style view | A delegate method supplies a pasteboard writer for the dragged item. | The session demonstrates NSTableView and notes similar methods for NSCollectionView, NSOutlineView, and NSBrowser. |
| MC.01.03.b030 | Text is selected in a custom view | Text selection can support bidirectional selection, text drag and drop, and toggling. | Attach NSTextSelectionManager to the view and configure a text-selection data source; the session identifies this as new in macOS 27. |
| MC.01.03.b031 | A registered control event occurs | AppKit calls the registered target and action. | Control events can respond to user-driven tracking state changes on standard controls without subclassing them. |
| MC.01.03.b032 | A gesture recognizer is added to a view hierarchy | An overlapping sibling view can block mouse events from reaching a control. | Resize the overlapping view, or override hitTest to return nil when an overlay should let hit testing pass through. |
| MC.01.03.b033 | The user presses Tab or Shift-Tab | Focus moves through controls in the window's key view loop. | Keyboard navigation for controls must be enabled in system settings. |
| MC.01.03.b034 | A view is added to or removed from a window's hierarchy | The key view loop is recalculated automatically. | Set autorecalculatesKeyViewLoop to true; otherwise, the app maintains the loop. |
| MC.01.03.b035 | The user presses Return while keyboard navigation focuses a status-item button | The status item's target action fires. | Configure the NSStatusItem button with a target and action. |
| MC.01.03.b036 | A status item opens custom UI | AppKit begins an expanded interface session so keyboard focus can be managed; the delegate shows the window when the session begins and orders it out when the session ends. | Set an expanded interface delegate and implement its begin and end methods. |
| MC.01.03.b037 | An action in a status item's expanded interface requests dismissal | The expanded interface session is canceled. | Call cancel on expandedInterfaceSession; the session may also be canceled when focus moves elsewhere. |
| MC.01.03.b038 | The app is asked to terminate while a sheet or modal window is open | The app may terminate without waiting for intervention, allowing more graceful termination. | Set preventsApplicationTerminationWhenModal to false for modals or sheets that do not strictly require user intervention. |
| MC.01.03.b039 | The app's UI state changes in a way that should be saved | The affected object's restorable state is invalidated and can later be encoded. | Call invalidateRestorableState when relevant UI state or hierarchy changes; encodeRestorableState is called when state has been invalidated. |
| MC.01.03.b040 | The app relaunches and windows need restoration | The restoration handler recreates each window and passes it to the completion handler; restored windows then decode their UI state. | Opt into restoration, provide window identifiers and a restoration class, and always call the completion handler, including on failure. |
| MC.01.03.b041 | A user clicks glass on a supported control | The glass subtly bounces, making the control feel responsive. | This is a macOS 27 effect intended for interactive controls, buttons, or containers of interactive controls. |
| MC.01.03.b042 | A view with configured concentric corners is near its container's corner | Its corner radius adapts to the container's shape. | Use cornerConfiguration with a container-concentric radius and, optionally, a minimum radius. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.01.03.k001 | Tab | Move focus through controls in the key view loop. | Keyboard navigation for controls is enabled. |
| MC.01.03.k002 | Shift-Tab | Move focus backward through controls in the key view loop. | Keyboard navigation for controls is enabled. |
| MC.01.03.k003 | Return | Trigger the focused status-item button's action. | The status-item button has a target and action. |

macOS 27 statements on this page:

- MC.01.03.n01: Liquid Glass continues to evolve; apps using it can receive several updates automatically when run on macOS 27.
- MC.01.03.n02: The automatic NSScrollEdgeEffectStyle resolves to a hard-edge effect around free-floating text, such as a window title in the title bar.
- MC.01.03.n03: Sidebars extend to window edges, their selection uses a semibold text style, and content continues to flow behind them.
- MC.01.03.n04: Bordered toolbar items over sidebars adopt Liquid Glass.
- MC.01.03.n05: A new interactive glass effect subtly bounces when clicked.
- MC.01.03.n06: NSTextSelectionManager is identified as a new macOS 27 API for bringing text-selection behavior to custom views.
- MC.01.03.n07: AppKit adds corner-concentricity APIs so views can adapt their corner radii to their containers.

API names: `mouseDown`, `NSCollectionViewItem.selected`, `NSTableRowView.selected`, `NSTableViewDelegate`, `NSOutlineViewDelegate`, `NSView.defaultMenu`, `NSResponder.menu`, `NSView.menuForEvent`, `NSTableView.tableView(_:pasteboardWriterForRow:)`, `NSCollectionView`, `NSOutlineView`, `NSBrowser`, `NSPasteboardItem`, `NSTextSelectionManager`, `NSControlEvents`, `NSControl.addTarget(_:action:for:)`, `NSGestureRecognizer`, `NSView.hitTest(_:)`, `NSWindow.autorecalculatesKeyViewLoop`, `NSStatusItem.button`, `NSStatusItem.view`, `NSStatusItem.expandedInterfaceDelegate`, `NSStatusItemExpandedInterfaceDelegate`, `NSStatusItemExpandedInterfaceSession`, `NSStatusItem.expandedInterfaceSession`, `NSWindow.preventsApplicationTerminationWhenModal`, `NSWindowRestoration`, `NSWindow.isRestorable`, `NSWindow.restorationClass`, `NSWindow.setFrameAutosaveName(_:)`, `NSResponder.encodeRestorableState(with:)`, `NSResponder.restoreState(with:)`, `NSResponder.invalidateRestorableState()`, `NSWindowRestoration.restoreWindow(withIdentifier:state:completionHandler:)`, `NSScrollEdgeEffectStyle`, `NSView.cornerConfiguration`, `NSViewCornerConfiguration`, `NSViewCornerRadius.containerConcentric`, `NSViewCornerConfiguration.uniformCorners`

System Settings paths referenced: System Settings (keyboard navigation for controls; the session does not specify a pane or exact path)

#### macOS 27 Golden Gate Release Notes

T1 · Release notes · [macOS 27 release notes](https://developer.apple.com/documentation/macos-release-notes/macos-27-release-notes) · macOS 27 text present: yes

Desktop UI–related notes cover menu-item image visibility, keyboard access to panel Recents, window movement and tiling for panels, toolbar and tab semantics, gesture behavior, and several UI fixes. The release notes do not describe changes to Control Center, widgets, or window traffic-light controls.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.01.03.e067 | Window | menu | Menu bar | Contains the “Move & Resize” and “Full Screen Tile” menus. |
| MC.01.03.e068 | Move & Resize | menu item | Window menu | The Window menu’s Move & Resize options can be used with open and save panels that are not displayed as sheets. |
| MC.01.03.e069 | Full Screen Tile | menu item | Window menu | The Window menu’s Full Screen Tile options can be used with open and save panels that are not displayed as sheets. |
| MC.01.03.e070 | Recents | view | Open and save panels | The Recents list can be accessed with Command-Shift-F. |
| MC.01.03.e071 | menu item images | other | Menu bar and context menus | The release notes describe reduced menu-item image display. AppKit and SwiftUI hide symbol images by default in most relevant menu contexts; AppKit behavior varies by linked SDK, and SwiftUI provides a label style to request an icon. |
| MC.01.03.e072 | NSTitlebarAccessoryViewController | view | Window title bar | When an app is linked on macOS 27.0 or later, title-bar accessory view controllers are allowed to draw outside their bounds by default. Accessories may be clipped during reveal animations or while hidden. |
| MC.01.03.e073 | NSToolbarItemGroup | control | Window toolbar | Adds a semantic role property and role enum for toolbar item groups. |
| MC.01.03.e074 | TabsPickerStyle | control | Pickers and tab-based navigation | A picker style for tab-based navigation and content selection. VoiceOver reads it as “tabs”; on macOS it has a distinct appearance from value-selection pickers. |
| MC.01.03.e075 | NSRefreshController | control | NSScrollView | Provides pull-to-refresh functionality for AppKit scroll views. |
| MC.01.03.e076 | PlayStation® Access™ controller | other | Game controller settings | Supported on macOS; custom input profiles can be created in game controller settings and saved to the device. |
| MC.01.03.e077 | Intel-based apps | view | System Settings > General | The view lists Intel-based apps that will be incompatible with macOS 28.0 and identifies unused Intel-based software found on the system. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.01.03.b043 | Press Command-Shift-F while using an open or save panel. | Accesses the panel’s Recents list. | macOS 27 AppKit feature. |
| MC.01.03.b044 | Use Window > Move & Resize or Window > Full Screen Tile. | The menu options can move, resize, open, or save panels. | Applies to open and save panels that are not displayed as sheets. |
| MC.01.03.b045 | Display menu-bar or context menus. | Menu-item images are shown less broadly: AppKit hides symbol images by default, and SwiftUI hides symbol images in most contexts. Common system-wide items such as Settings, Share, and Print can receive default visible images. | AppKit behavior depends on the app’s linked SDK and menu-item configuration. SwiftUI apps can request an icon with labelStyle(.titleAndIcon); AppKit apps can use NSMenuItem.preferredImageVisibility. |
| MC.01.03.b046 | Use an NSTitlebarAccessoryViewController in an app linked on macOS 27.0 or later. | The accessory can draw outside its bounds by default, supporting effects such as shadows and interactive glass effects. | Accessories may be clipped during reveal animations or while hidden. |
| MC.01.03.b047 | Use a tab-oriented segmented control or picker. | VoiceOver identifies the control as tabs, and macOS gives it a distinct appearance from controls used for value selection. | Use the relevant tab role or TabsPickerStyle. |
| MC.01.03.b048 | Pan an enclosing scroll view while a gesture recognizer is active. | The recognizer can be automatically cancelled. | Configure NSGestureRecognizer.cancellableByScrollGesture. |
| MC.01.03.b049 | User input ends while an exclusive gesture remains stuck. | AppKit automatically cancels the stuck gesture after a few seconds. | NSCrashOnStuckGestureTimeout can be enabled to crash the app for debugging. |
| MC.01.03.b050 | Use Mission Control with an app in full screen. | A Dock persistence issue when entering or exiting Mission Control is fixed. | Resolved issue in macOS 27 release notes. |
| MC.01.03.b051 | Use an open or save panel with keyboard focus or tab navigation. | A previously reported issue that could prevent traversal through all fields and controls, including the accessory view, is fixed. | Resolved AppKit issue. |
| MC.01.03.b052 | Use a SwiftUI TextField with a custom prompt font or color. | In apps built with the macOS 27.0 SDK, the TextField respects styling applied to the Text supplied as its prompt. | Apply the styling to the prompt Text at initialization. |
| MC.01.03.b053 | Use a SwiftUI List as a drop destination. | In apps built with the 27.0 SDKs, compatible drops can be accepted into reorderable content without the .reorderableItem transfer type, and a dropDestination modifier on a list item performs the drop. | macOS List behavior is aligned with existing iOS List behavior. |
| MC.01.03.b054 | Use a SwiftUI sheet or popover containing controls. | In apps built with the 27.0 SDKs, several control-related environment values are reset to their defaults in the sheet or popover. | Applies to controlSize, buttonSizing, buttonRepeatBehavior, menuIndicatorVisibility, and ButtonBorderShape. |
| MC.01.03.b055 | Use a SwiftUI disabled checkbox-style Toggle. | It is no longer tinted in macOS, making its disabled state easier to distinguish. | SwiftUI .checkbox style. |
| MC.01.03.b056 | Open a window from Mission Control after exiting full-screen mode. | A reported issue that could cause the window to be lost is fixed. | Resolved Window Management issue. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.01.03.k004 | Command-Shift-F | Accesses the Recents list. | Open and save panels. |

macOS 27 statements on this page:

- MC.01.03.n01: The Recents list in the open and save panels can be accessed with the keyboard shortcut Command-Shift-F.
- MC.01.03.n02: Window > Move & Resize and Window > Full Screen Tile menus can be used to move, resize, open and save panels that are not displayed as a sheet.
- MC.01.03.n03: In macOS 27.0, AppKit adds NSRefreshController, providing pull-to-refresh functionality for NSScrollView.
- MC.01.03.n04: NSToolbarItemGroup adds the role property and NSToolbarItemGroupRole enum. NSSegmentedControl similarly adds a role property and NSSegmentedControlRole enum, including a tabs role.
- MC.01.03.n05: In macOS 27.0, menu bar and context menus present a reduced set of menu item images. By default, NSMenu hides all menu item symbol images; non-symbol images remain visible. NSMenuItem.preferredImageVisibility can customize visibility.
- MC.01.03.n06: For applications linked on the macOS 27 SDK, both symbol and non-symbol menu item images are automatically hidden. Applications linked on earlier SDKs retain the earlier behavior for non-symbol images. Use preferredImageVisibility if images need to remain visible.
- MC.01.03.n07: For applications linked on SDKs prior to macOS 27, NSMenu automatically shows menu item images when both the menu item title and attributed title are empty. With the macOS 27 SDK, these images are automatically hidden unless visibility is customized.
- MC.01.03.n08: NSTextView’s Layout Orientation menu item and submenu are located inside the Font submenu only for applications linked on the macOS 27 SDK.
- MC.01.03.n09: When linked on macOS 27.0 or later, NSTitlebarAccessoryViewController is allowed to draw outside its bounds by default.
- MC.01.03.n10: The Dock issue where it might persist when entering or exiting Mission Control while using an app in full screen is fixed.
- MC.01.03.n11: The Dock issue where it might incorrectly indicate that some applications are running in the background is fixed.
- MC.01.03.n12: Finder’s suggested file and folder names are fixed to be more specific to their contents in the reported case.
- MC.01.03.n13: Settings > General now lists Intel-based apps that will be incompatible with macOS 28.0 and identifies unused Intel-based software discovered on the system. The system might suggest a website for an Apple silicon version.
- MC.01.03.n14: Keyboard focus and tab order in expanded and collapsed open/save panels are fixed so traversal can include all fields and controls, including the accessory view.
- MC.01.03.n15: NSAlert buttons no longer erroneously allow bezelColor customization.
- MC.01.03.n16: NSScrollView no longer displays the system scroll-edge effect when it has no visible scrollers.
- MC.01.03.n17: A reported Liquid Glass rendering issue for NSSegmentedCell, where the control appeared at the wrong location, is fixed.
- MC.01.03.n18: When full keyboard navigation is off, the keyboard loop is fixed so it can traverse the correct fields and controls.
- MC.01.03.n19: In apps built with the macOS 27.0 SDK, TextField respects custom font and color styling applied to its prompt.
- MC.01.03.n20: Bordered SwiftUI Menu and Picker buttons support better label customization and no longer use NSPopUpButton in their implementation.
- MC.01.03.n21: In apps built with the 27.0 SDKs, selectable Text views support TextRenderer.
- MC.01.03.n22: SwiftUI hides menu-item symbol images by default in most menu contexts. Use labelStyle(.titleAndIcon) to indicate that a menu Label’s icon should always be shown.
- MC.01.03.n23: In apps built with the latest SDKs, TabViews in inspectors match their appearance in sidebars and automatically use the .tabs picker style.
- MC.01.03.n24: Disabled SwiftUI Toggles using the .checkbox style are no longer tinted in macOS.
- MC.01.03.n25: TabsPickerStyle is available for pickers representing tab-based navigation and content selection; VoiceOver reads them as “tabs,” with a distinct macOS appearance.
- MC.01.03.n26: SwiftUI TextField border shapes can be customized with TextInputBorderShape and textInputBorderShape(_:). The .squareBorder and .roundedBorder text field styles are soft deprecated in favor of .bordered.
- MC.01.03.n27: On macOS, the SwiftUI Slider implementation no longer uses NSSlider.
- MC.01.03.n28: In apps built with the 27.0 SDKs, LabeledContent inside a Menu maps its value to the platform menu item’s subtitle.
- MC.01.03.n29: The new data-item or error-object based alert and confirmationDialog modifiers can be used by projects targeting macOS 12.0 and later.
- MC.01.03.n30: In macOS apps built with the macOS 27 SDK, the action from the newDocument environment value can accept an in-memory ReadableDocument and present a new document window populated with it.
- MC.01.03.n31: Fixed: Some control-related environment values are reset to their defaults in sheets and popovers in apps built with the 27.0 SDKs.
- MC.01.03.n32: Fixed: On macOS, buttons using the .glass and .glassProminent styles display a hover state when used outside a toolbar.
- MC.01.03.n33: In macOS apps built with the macOS 27 SDK, List accepts specified drops that previously did not work, including drops handled by a dropDestination modifier on a list item.
- MC.01.03.n34: Fixed: The “Settings…” menu item displays an icon.
- MC.01.03.n35: The .disableScreenCornerInteractions presentation option is available to disable Hot Corners.
- MC.01.03.n36: NSGestureRecognizer adds cancellableByScrollGesture. NSView adds exclusiveGestureBehavior; NSViewGestureRecognizerIsExclusive and NSGestureRecognizerSuppressesMainMenuActions provide app-level gesture behavior controls.
- MC.01.03.n37: Exclusive gestures are automatically cancelled after user input has ended for a few seconds. NSCrashOnStuckGestureTimeout can make the app crash in this condition for debugging.
- MC.01.03.n38: NSGestureRecognizerCrashOnMissingOverrides is a diagnostic user default for finding gesture-recognizer subclasses missing required overrides.
- MC.01.03.n39: NSScrollView adds properties to constrain the number of touches needed to scroll and to set gesture relationships against scrolling gestures.
- MC.01.03.n40: The Intel-app compatibility list is also described under Settings > General; the release notes say listed Intel-based apps will be incompatible with macOS 28.0.

API names: `NSRefreshController`, `NSScrollView.refresh`, `beginRefreshing`, `endRefreshing`, `NSToolbarItemGroup.role`, `NSToolbarItemGroupRole`, `NSSegmentedControl.role`, `NSSegmentedControlRole`, `NSMenuItem.preferredImageVisibility`, `NSTextView.menuForEvent:`, `NSTitlebarAccessoryViewController`, `TextRenderer`, `Text.textSelection`, `labelStyle(_:)`, `.titleAndIcon`, `TabViews`, `TabsPickerStyle`, `.tabs`, `TextInputBorderShape`, `textInputBorderShape(_:)`, `.squareBorder`, `.roundedBorder`, `.bordered`, `LabeledContent`, `NSApplication.presentationOptions`, `.disableScreenCornerInteractions`, `NSGestureRecognizer.cancellableByScrollGesture`, `NSView.exclusiveGestureBehavior`, `NSViewGestureRecognizerIsExclusive`, `NSGestureRecognizerSuppressesMainMenuActions`, `NSCrashOnStuckGestureTimeout`, `NSGestureRecognizerCrashOnMissingOverrides`, `NSScrollView.scrollGestureForFailureRelationship`, `TextField`, `Menu`, `Picker`, `Slider`, `List`, `.reorderableItem`, `.dropDestination`, `.glass`, `.glassProminent`, `alert`, `confirmationDialog`, `ReadableDocument`, `newDocument`

System Settings paths referenced: System Settings > General; Privacy & Security settings; game controller settings

#### Independent reports (MC.01.03)

| ID | Kind | Area | Claim | Source |
|---|---|---|---|---|
| MC.01.03.x01 | changed | menus | In the macOS 27 Golden Gate developer beta, many menu item icons found in macOS 26 Tahoe have been removed; icons remain only where they are considered useful. | T4 [MacRumors](https://www.macrumors.com/2026/06/11/macos-27-golden-gate-menu-items/) |
| MC.01.03.x02 | behavior | menus | macOS 26 Tahoe was the first macOS version to place a small icon next to nearly every menu entry in Apple's apps. | T4 [MacRumors](https://www.macrumors.com/2026/06/11/macos-27-golden-gate-menu-items/) |
| MC.01.03.x03 | behavior | menus | Apple's revised Human Interface Guidelines advise developers to use menu item icons sparingly and with purpose, reserving them for common actions, file system locations, connected devices, and similar cases. | T4 [MacRumors](https://www.macrumors.com/2026/06/11/macos-27-golden-gate-menu-items/) |
| MC.01.03.x04 | changed | menus | macOS 27 Golden Gate removes the icons that macOS 26 Tahoe displayed next to menu items. | T4 [Daring Fireball](https://daringfireball.net/2026/06/macos_27_golden_gate_removes_the_dumb_icons_from_menu_items) |
| MC.01.03.x05 | behavior | menus | In macOS 26 Tahoe, icons appeared next to menu items across Apple apps; the article says different Apple apps used different icons for the same menu items. | T4 [Daring Fireball](https://daringfireball.net/2026/06/macos_27_golden_gate_removes_the_dumb_icons_from_menu_items) |
| MC.01.03.x06 | changed | menus | Apple's updated Human Interface Guidelines advise using menu item icons sparingly and with purpose, and omitting them when no clear representation is available. | T4 [Daring Fireball](https://daringfireball.net/2026/06/macos_27_golden_gate_removes_the_dumb_icons_from_menu_items) |
| MC.01.03.x07 | behavior | menus | The updated guidelines say icons can help people find menu items more quickly and clarify what selecting an item does. They recommend them for common actions and key features, file system locations, connected devices, visual concepts, and user-generated content. | T4 [Daring Fireball](https://daringfireball.net/2026/06/macos_27_golden_gate_removes_the_dumb_icons_from_menu_items) |
| MC.01.03.x08 | workaround | menus | Third-party developers used open-source code to disable the default behavior of showing icons in standard menu items. | T4 [Daring Fireball](https://daringfireball.net/2026/06/macos_27_golden_gate_removes_the_dumb_icons_from_menu_items) |
| MC.01.03.x09 | changed | menus | In macOS 27, menus display almost no icons, returning to a presentation similar to that of older versions of macOS. (translated) | T4 [MacGeneration](https://www.macg.co/macos/2026/06/la-barre-des-menus-de-macos-27-masque-automatiquement-ses-icones-309075) |
| MC.01.03.x10 | behavior | menus | Menu icons are hidden by default, except when an app’s developer explicitly chooses to show them to highlight them. (translated) | T4 [MacGeneration](https://www.macg.co/macos/2026/06/la-barre-des-menus-de-macos-27-masque-automatiquement-ses-icones-309075) |
| MC.01.03.x11 | removed | menus | There is no option to keep the macOS 26 presentation, where every menu item was accompanied by an icon. (translated) | T4 [MacGeneration](https://www.macg.co/macos/2026/06/la-barre-des-menus-de-macos-27-masque-automatiquement-ses-icones-309075) |
| MC.01.03.x12 | behavior | menus | The changes to menu icons are also available on iPad. (translated) | T4 [MacGeneration](https://www.macg.co/macos/2026/06/la-barre-des-menus-de-macos-27-masque-automatiquement-ses-icones-309075) |
| MC.01.03.x13 | changed | menus | In macOS 26, mini-icons were added to the left of entries in the drop-down text menus in the menu bar. (translated) | T4 [iFun](https://www.ifun.de/finder-menues-in-macos-27-wieder-ohne-mini-symbole-281385/) |
| MC.01.03.x14 | changed | menus | In the first developer beta of macOS 27, the mini-icons were removed from Finder menus. (translated) | T4 [iFun](https://www.ifun.de/finder-menues-in-macos-27-wieder-ohne-mini-symbole-281385/) |
| MC.01.03.x15 | behavior | menus | According to the article, there seems to be no setting planned that would let users show the removed icons again. (translated) | T4 [iFun](https://www.ifun.de/finder-menues-in-macos-27-wieder-ohne-mini-symbole-281385/) |
| MC.01.03.x16 | behavior | menus | AppKit apps linked with the macOS 26.0 SDK or later hide menu-item symbol images by default and display non-symbol images. (translated) | T4 [Blake Crosley](https://blakecrosley.com/ko/blog/menu-images-hidden-macos-27) |
| MC.01.03.x17 | changed | menus | AppKit apps linked with the macOS 27 SDK hide both symbol and non-symbol menu-item images by default. The behavior depends on the SDK the app is linked with. (translated) | T4 [Blake Crosley](https://blakecrosley.com/ko/blog/menu-images-hidden-macos-27) |
| MC.01.03.x18 | behavior | menus | The menu image changes described in the article do not apply to apps linked with an SDK earlier than macOS 26. (translated) | T4 [Blake Crosley](https://blakecrosley.com/ko/blog/menu-images-hidden-macos-27) |
| MC.01.03.x19 | changed | menus | In apps linked with an SDK earlier than macOS 27, an image is automatically displayed if both the menu item’s title and attributed title are empty. When linked with the macOS 27 SDK, this safeguard is removed, so the image may be hidden. (translated) | T4 [Blake Crosley](https://blakecrosley.com/ko/blog/menu-images-hidden-macos-27) |
| MC.01.03.x20 | behavior | menus | A menu item that contains only an image can remain in the menu and respond to clicks even when the image is hidden, so it may look like an empty row. (translated) | T4 [Blake Crosley](https://blakecrosley.com/ko/blog/menu-images-hidden-macos-27) |
| MC.01.03.x21 | behavior | menus | Even when an image is hidden, the menu’s height, separators, and clickable areas may remain unchanged. UI tests that only check the number of items can miss rendering problems, while screenshot comparisons can catch them. (translated) | T4 [Blake Crosley](https://blakecrosley.com/ko/blog/menu-images-hidden-macos-27) |
| MC.01.03.x22 | workaround | menus | AppKit provides `preferredImageVisibility` to control image visibility for each menu item; its values are `.automatic`, `.visible`, and `.hidden`. The article recommends setting `.visible` to keep images in the macOS 27 SDK. (translated) | T4 [Blake Crosley](https://blakecrosley.com/ko/blog/menu-images-hidden-macos-27) |
| MC.01.03.x23 | behavior | menus | For menu items created in a xib, the “macOS 26.0 only” checkbox in the menu item inspector affects whether the image is displayed. If unchecked, the image remains visible; if checked, it is hidden. (translated) | T4 [Blake Crosley](https://blakecrosley.com/ko/blog/menu-images-hidden-macos-27) |
| MC.01.03.x24 | behavior | menus | AppKit continues to display default images for commonly used system-wide menu items such as “설정” (Settings), “공유” (Share), and “프린트” (Print). (translated) | T4 [Blake Crosley](https://blakecrosley.com/ko/blog/menu-images-hidden-macos-27) |
| MC.01.03.x25 | changed | menus | The UIKit release notes explain that, by default, images set on menu elements are not displayed in menu bars and context menus in iPadOS 27.0 and macOS 27.0. (translated) | T4 [Blake Crosley](https://blakecrosley.com/ko/blog/menu-images-hidden-macos-27) |
| MC.01.03.x26 | workaround | menus | UIKit provides `preferredImageVisibility` to control the display of menu images, with values `.automatic`, `.visible`, and `.hidden`. The article also mentions updated initializers for `UIMenu`, `UIAction`, `UICommand`, and `UIKeyCommand`. (translated) | T4 [Blake Crosley](https://blakecrosley.com/ko/blog/menu-images-hidden-macos-27) |
| MC.01.03.x27 | behavior | menus | SwiftUI hides symbol images for menu items by default in most contexts, while continuing to display non-symbol images. The article describes this default behavior as matching AppKit’s baseline. (translated) | T4 [Blake Crosley](https://blakecrosley.com/ko/blog/menu-images-hidden-macos-27) |
| MC.01.03.x28 | workaround | menus | To display an icon for a SwiftUI menu item, you can use `.labelStyle(.titleAndIcon)` on a `Label`. (translated) | T4 [Blake Crosley](https://blakecrosley.com/ko/blog/menu-images-hidden-macos-27) |
| MC.01.03.x29 | behavior | menus | According to the article, the SwiftUI release notes mention menu bars in iPadOS 27.0 and macOS 27.0, and context menus in macOS 27.0. The notes do not mention SwiftUI context-menu behavior on iPadOS. (translated) | T4 [Blake Crosley](https://blakecrosley.com/ko/blog/menu-images-hidden-macos-27) |
| MC.01.03.x30 | behavior | menus | SwiftUI also continues to display default images for common system menu items such as “설정” (Settings), “공유” (Share), and “프린트” (Print). (translated) | T4 [Blake Crosley](https://blakecrosley.com/ko/blog/menu-images-hidden-macos-27) |
| MC.01.03.x31 | changed | menus | System apps use fewer SF Symbols in menus, and Apple’s Human Interface Guidelines advise third-party developers to use them “sparingly and with purpose.” | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/2/) |
| MC.01.03.x32 | new | menus | An "Ask Siri" entry appears at the top of every contextual menu, regardless of whether it applies, and is darker than other menu items. | T4 [Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/) |
| MC.01.03.x33 | behavior | menus | Selecting "Ask Siri" brings a text-entry box forward and dismisses the rest of the contextual menu. | T4 [Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/) |
| MC.01.03.x34 | behavior | menus | The Ice Bar displays item glyphs cut from a capture of the menu bar, with their backgrounds removed and the glyphs re-tinted; clicking an item opens that item's interface. | T2 [GitHub jordanbaird/Ice](https://github.com/jordanbaird/Ice/pull/995) |
| MC.01.03.x35 | workaround | menus | For an item in the Ice Bar whose menu-bar item is concealed, Ice temporarily allows its application, clicks the item when it is drawn, then hides it again after its menu closes. | T2 [GitHub jordanbaird/Ice](https://github.com/jordanbaird/Ice/pull/995) |
| MC.01.03.x36 | workaround | menus | The pull request reports that clicks on the clock, battery, or Wi-Fi are ignored while an assertion is active; Ice lifts concealment and replays the click. A click on Control Centre is instead sent through Accessibility without lifting concealment. | T2 [GitHub jordanbaird/Ice](https://github.com/jordanbaird/Ice/pull/995) |
| MC.01.03.x37 | changed | menus | In Beta Four, the delay between clicking an item in Bartender Bar and its menu opening was reduced; clicking individual Stats widgets should also be more reliable. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.03.x38 | new | menus | In Beta Three, right-clicking empty menu bar space opens Bartender's menu. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.03.x39 | changed | menus | The post’s overview says icons have been removed from every menu item. The SDK notes clarify that macOS 27 hides menu-item symbol images by default, while non-symbol images remain visible; certain system-wide items, including Settings, Share, and Print, receive default visible images. The SDK notes say this applies to apps linked on macOS 26.0 and later. | T5 [MacRumors Forums](https://forums.macrumors.com/threads/macos-27-all-the-little-things.2483520/) |
| MC.01.03.x40 | changed | menus | The post says menu bar and context menus present a reduced set of menu-item images, similar to behavior before macOS 26.0. For SwiftUI menu labels, the SDK notes identify `.titleAndIcon` as the style for keeping an icon visible. | T5 [MacRumors Forums](https://forums.macrumors.com/threads/macos-27-all-the-little-things.2483520/) |
| MC.01.03.x41 | new | menus | Right-clicking folders, files, or apps often shows an “Ask Siri” option at the top, allowing questions about the selected item. | T4 [Tom's Guide](https://www.tomsguide.com/computing/macos/macos-golden-gate-review) |
| MC.01.03.x42 | changed | menus | In macOS 27, menu bars and context menus present fewer menu item images. By default, NSMenu hides symbol images; non-symbol images remain visible. This behavior applies to apps linked on macOS 26.0 and later. | T4 [BetaProfile](https://betaprofile.dev/macos-27-rc/) |
| MC.01.03.x43 | behavior | menus | For menu items created from a xib file, image visibility also depends on the “macOS 26.0 only” checkbox in the menu item inspector: unchecked keeps the image visible, while checked hides it. | T4 [BetaProfile](https://betaprofile.dev/macos-27-rc/) |
| MC.01.03.x44 | changed | menus | Apps linked on the macOS 27 SDK automatically hide both symbol and non-symbol menu item images. Apps linked on earlier SDKs retain automatic visibility for non-symbol images. Apps can use preferredImageVisibility to keep images visible when needed. | T4 [BetaProfile](https://betaprofile.dev/macos-27-rc/) |
| MC.01.03.x45 | changed | menus | For apps linked on SDKs before macOS 27, NSMenu automatically shows an image when both the menu item title and attributed title are empty. With the macOS 27 SDK, those images are automatically hidden unless visibility is configured. | T4 [BetaProfile](https://betaprofile.dev/macos-27-rc/) |
| MC.01.03.x46 | changed | menus | SwiftUI hides menu item symbol images by default in most contexts, while non-symbol images remain visible. Use labelStyle(_:) with .titleAndIcon to indicate that a Label’s icon should always be shown. | T4 [BetaProfile](https://betaprofile.dev/macos-27-rc/) |
| MC.01.03.x47 | behavior | menus | The system automatically provides visible images for certain common menu items, including Settings, Share, and Print. | T4 [BetaProfile](https://betaprofile.dev/macos-27-rc/) |
| MC.01.03.x48 | changed | menus | For apps linked on the macOS 27 SDK, NSTextView’s context menu places Layout Orientation and its submenu inside the Font submenu. Apps linked on earlier SDKs keep Layout Orientation at the top level. | T4 [BetaProfile](https://betaprofile.dev/macos-27-rc/) |
| MC.01.03.x49 | new | menus | The Window > Move & Resize and Window > Full Screen Tile menus can move or resize open and save panels that are not displayed as sheets. | T4 [BetaProfile](https://betaprofile.dev/macos-27-rc/) |
| MC.01.03.x50 | changed | menus | In macOS 27 Golden Gate, most menus contain only a few icons; only certain menu items have icons, while others remain text-only. The article says this reduces visual clutter and makes menu options easier to distinguish at a glance. | T4 [TechRadar](https://www.techradar.com/computing/glaringly-inconsistent-and-often-utterly-inscrutable-macos-27-golden-gate-just-fixed-one-of-my-biggest-macos-tahoe-gripes) |
| MC.01.03.x51 | behavior | menus | In macOS Tahoe, app menus were often filled with icons, sometimes reusing the same icon for different neighboring menu items, making items harder to distinguish. | T4 [TechRadar](https://www.techradar.com/computing/glaringly-inconsistent-and-often-utterly-inscrutable-macos-27-golden-gate-just-fixed-one-of-my-biggest-macos-tahoe-gripes) |
| MC.01.03.x52 | behavior | menus | Apple’s third-party design guidelines advise using menu items sparingly and with purpose, and using icons to highlight common actions and key features. The article says an item should not get an icon if it does not fit an existing one. | T4 [TechRadar](https://www.techradar.com/computing/glaringly-inconsistent-and-often-utterly-inscrutable-macos-27-golden-gate-just-fixed-one-of-my-biggest-macos-tahoe-gripes) |
| MC.01.03.x53 | changed | menus | Most icons alongside menu commands have been removed, returning menus to a less visually cluttered appearance. | T4 [9to5Mac](https://9to5mac.com/2026/09/14/macos-27-golden-gate-now-available-here-is-everything-new/) |
| MC.01.03.x54 | new | menus | Context menus have a systemwide “Ask Siri” option for asking about selected text, images, or files. | T4 [9to5Mac](https://9to5mac.com/2026/09/14/macos-27-golden-gate-now-available-here-is-everything-new/) |

### MC.01.04 Status menus / menu extras

Status: **PARTIAL**. Apple documents Command-drag rearranging; macOS 27 overflow control and MenuBarAgent changes are third-party evidence only (GAP-B5-03, 04).

Also covered by: [Customize the menu bar on Mac](https://support.apple.com/guide/mac-help/mchl4af84660/27/mac/27), [Change Menu Bar settings on Mac](https://support.apple.com/guide/mac-help/mchlad96d366/27/mac/27), [What’s in the menu bar on Mac?](https://support.apple.com/guide/mac-help/mchlp1446/27/mac/27), [The menu bar](https://developer.apple.com/design/human-interface-guidelines/the-menu-bar)


#### Independent reports (MC.01.04)

| ID | Kind | Area | Claim | Source |
|---|---|---|---|---|
| MC.01.04.x01 | changed | menu bar | Beta 6 thins menu bars and removes unnecessary icons. | T4 [iThinkDiff](https://www.ithinkdiff.com/macos-golden-gate-beta-6-wallpapers-traffic-light-buttons/) |
| MC.01.04.x02 | new | menu bar | When there isn’t enough space in the menu bar, macOS prioritizes menus and automatically hides the icons on the right that no longer fit. (translated) | T4 [MacGeneration](https://www.macg.co/macos/2026/06/la-barre-des-menus-de-macos-27-masque-automatiquement-ses-icones-309075) |
| MC.01.04.x03 | behavior | menu bar | The hidden icons are replaced by two chevrons. Clicking them collapses the menus to show all the hidden icons. (translated) | T4 [MacGeneration](https://www.macg.co/macos/2026/06/la-barre-des-menus-de-macos-27-masque-automatiquement-ses-icones-309075) |
| MC.01.04.x04 | behavior | menu bar | The icons revealed by the chevrons remain usable as usual. Once you’re done using them, the menus return to their place. (translated) | T4 [MacGeneration](https://www.macg.co/macos/2026/06/la-barre-des-menus-de-macos-27-masque-automatiquement-ses-icones-309075) |
| MC.01.04.x05 | behavior | menu bar | In the first beta, there is no visible setting for controlling this behavior: it is managed automatically by the system. (translated) | T4 [MacGeneration](https://www.macg.co/macos/2026/06/la-barre-des-menus-de-macos-27-masque-automatiquement-ses-icones-309075) |
| MC.01.04.x06 | bug | menu bar | In the first beta, a bug still lets you click menu items behind the icons, even though those items are supposed to be hidden. (translated) | T4 [MacGeneration](https://www.macg.co/macos/2026/06/la-barre-des-menus-de-macos-27-masque-automatiquement-ses-icones-309075) |
| MC.01.04.x07 | changed | menu bar | Menu bar items no longer display icons by default. | T3 [Wikipedia](https://en.wikipedia.org/wiki/MacOS_Golden_Gate) |
| MC.01.04.x08 | new | menu bar | When the notch or an app’s menu system covers menu bar items, a double-arrow control appears. Clicking it temporarily reveals the hidden items to the left of the notch so they can be used. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/3/) |
| MC.01.04.x09 | new | menu bar | A new Ethernet icon replaces the Wi-Fi icon in the menu bar when the Mac is wired to a network. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/2/) |
| MC.01.04.x10 | new | menu bar | macOS adds menu bar item management to handle icons that overflow the notch on some MacBook models; clicking the chevron moves icons over the notch. | T4 [Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/) |
| MC.01.04.x11 | behavior | menu bar | Stage Manager can be added to the menu bar through Control Center's Edit Controls, under Desktop & Finder. | T4 [Lets Suite](https://www.lets-suite.com/blog/how-to-use-stage-manager-mac) |
| MC.01.04.x12 | bug | menu bar | In OnlySwitch 2.7.4 on macOS 27, toggling the hide-icons control made its arrow icon disappear and reappear, but did not hide the menu bar icons to its left. The reporter said the feature had worked on macOS 26.6.2. | T2 [GitHub jacklandrin/OnlySwitch](https://github.com/jacklandrin/OnlySwitch/issues/218) |
| MC.01.04.x13 | changed | menu bar | The OnlySwitch maintainer said Apple's menu bar redesign made the previous implementation unable to support hiding menu bar icons, and that version 2.7.5 partially fixed the feature while leaving some problems. | T2 [GitHub jacklandrin/OnlySwitch](https://github.com/jacklandrin/OnlySwitch/issues/218) |
| MC.01.04.x14 | bug | menu bar | After updating to version 2.7.5, the reporter said hiding menu items mostly worked again, but the arrow icon remained visible. | T2 [GitHub jacklandrin/OnlySwitch](https://github.com/jacklandrin/OnlySwitch/issues/218) |
| MC.01.04.x15 | behavior | menu bar | After updating, menu items no longer hid automatically on boot or reboot; the reporter said they had to right-click to hide them. They noted that enabling “Hide Menu after Running” might resolve this. | T2 [GitHub jacklandrin/OnlySwitch](https://github.com/jacklandrin/OnlySwitch/issues/218) |
| MC.01.04.x16 | behavior | menu bar | The reporter said an iPhone Live Activity showing a sports score was also hidden when menu icons were hidden, and could not be moved by holding Cmd and dragging. | T2 [GitHub jacklandrin/OnlySwitch](https://github.com/jacklandrin/OnlySwitch/issues/218) |
| MC.01.04.x17 | changed | menu bar | The reporter noted that macOS has a new double-left-arrow menu icon that can push icons to the left side of the notch. | T2 [GitHub jacklandrin/OnlySwitch](https://github.com/jacklandrin/OnlySwitch/issues/218) |
| MC.01.04.x18 | behavior | menu bar | The maintainer said Now Playing was affected too, because Apple does not expose the system icon IDs for it or Notification Center, even through private API. | T2 [GitHub jacklandrin/OnlySwitch](https://github.com/jacklandrin/OnlySwitch/issues/218) |
| MC.01.04.x19 | bug | menu bar | A commenter on macOS 27 reported that the divider icon remained visible even when the other icons were hidden. | T2 [GitHub jacklandrin/OnlySwitch](https://github.com/jacklandrin/OnlySwitch/issues/218) |
| MC.01.04.x20 | changed | menu bar | The pull request reports that macOS 27 replaced separate menu-bar item windows with a single bar drawn by `MenuBarAgent`; `CGWindowList` no longer provides the items, their frames, or their owners. | T2 [GitHub jordanbaird/Ice](https://github.com/jordanbaird/Ice/pull/995) |
| MC.01.04.x21 | behavior | menu bar | Ice reads menu-bar items, frames, owners, and the overflow control through Accessibility on macOS 27. | T2 [GitHub jordanbaird/Ice](https://github.com/jordanbaird/Ice/pull/995) |
| MC.01.04.x22 | behavior | menu bar | The pull request says macOS 27 can hide items only through private assessment-mode assertions, and that assertions combine as a union of allowlists. | T2 [GitHub jordanbaird/Ice](https://github.com/jordanbaird/Ice/pull/995) |
| MC.01.04.x23 | behavior | menu bar | While an assessment-mode assertion is active, `MenuBarAgent` ignores clicks on its own items; the pull request reports the same restriction for keyboard shortcuts such as Globe+N to open Notification Center. | T2 [GitHub jordanbaird/Ice](https://github.com/jordanbaird/Ice/pull/995) |
| MC.01.04.x24 | behavior | menu bar | Accessibility can continue reporting concealed items at the frames where they were last drawn, so the implementation tracks concealed items by process rather than inferring concealment from their frames. | T2 [GitHub jordanbaird/Ice](https://github.com/jordanbaird/Ice/pull/995) |
| MC.01.04.x25 | changed | menu bar | On macOS 27, `MenuBarAgent` reorders items itself; the order shown on the bar does not identify their sections, which are instead stored per application in the saved layout. | T2 [GitHub jordanbaird/Ice](https://github.com/jordanbaird/Ice/pull/995) |
| MC.01.04.x26 | behavior | menu bar | The pull request says application menus are not hidden on macOS 27; the system handles overflowing items behind its own control. | T2 [GitHub jordanbaird/Ice](https://github.com/jordanbaird/Ice/pull/995) |
| MC.01.04.x27 | behavior | menu bar | The pull request says items cannot be reordered directly on the macOS 27 menu bar; macOS arranges them, while the saved layout determines their sections. | T2 [GitHub jordanbaird/Ice](https://github.com/jordanbaird/Ice/pull/995) |
| MC.01.04.x28 | behavior | menu bar | When an application section is dragged to another row in the Menu Bar Layout editor, the application moves to that section and the saved layout is applied immediately. | T2 [GitHub jordanbaird/Ice](https://github.com/jordanbaird/Ice/pull/995) |
| MC.01.04.x29 | behavior | menu bar | Opening the Menu Bar Layout window temporarily shows all items so they can be photographed for the Ice Bar. | T2 [GitHub jordanbaird/Ice](https://github.com/jordanbaird/Ice/pull/995) |
| MC.01.04.x30 | workaround | menu bar | When an assessment-mode assertion is active, the system's camera and microphone indicators are not drawn; Ice instead draws an indicator while something is concealed and lets it be clicked to open Control Centre. | T2 [GitHub jordanbaird/Ice](https://github.com/jordanbaird/Ice/pull/995) |
| MC.01.04.x31 | behavior | menu bar | The pull request says Ice's camera and microphone indicator can be disabled in Advanced settings. | T2 [GitHub jordanbaird/Ice](https://github.com/jordanbaird/Ice/pull/995) |
| MC.01.04.x32 | behavior | menu bar | A follow-up reports that the application-specific “Allow in the Menu Bar” switch is shown in System Settings → Menu Bar. The contributor says changing it hides or restores an application's items, but the pull-request author reports that the proposed preference could not be found or verified on their Mac. | T2 [GitHub jordanbaird/Ice](https://github.com/jordanbaird/Ice/pull/995) |
| MC.01.04.x33 | behavior | menu bar | On macOS 27, some apps' menu bar items cannot be hidden independently: an item in Hidden can remain visible if another item from the same app is in Visible. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x34 | behavior | menu bar | On macOS 27, macOS hides Focus, AirDrop, Now Playing, and Fast User Switching whenever another item is hidden, and does not hide Time Machine or the Input menu. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x35 | behavior | menu bar | On macOS 27, the Clock, Control Center, and Siri are pinned and Thaw cannot move them. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x36 | behavior | menu bar | On macOS 27, the Shortcuts item can be hidden at launch, and the Focus icon may not be visible to Thaw; Thaw cannot manage either item yet. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x37 | behavior | menu bar | On macOS 27, macOS hides some Live Activity items and Thaw cannot bring them back on demand. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x38 | behavior | menu bar | Apps built for the older menu bar can appear in the wrong place, refuse to hide, or show no icon; the app developer must fix this. The page gives f.lux as an example. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x39 | behavior | menu bar | On macOS 27, items with changing text, such as a temperature, clock, or transfer rate, can appear next to their assigned position rather than exactly there; macOS places them from memory. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x40 | behavior | menu bar | On macOS 27, apps add their own padding around menu bar items, so the same spacing setting can leave uneven gaps. Applying spacing can also restart menu bar apps. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x41 | behavior | menu bar | On macOS 27, when Thaw has no usable capture of an item, including when Screen Recording is off, it displays the owning app's icon instead of a preview. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x42 | workaround | menu bar | On macOS 27, Thaw cannot see Little Snitch's menu bar icon until Little Snitch allows GUI scripting. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x43 | behavior | menu bar | On macOS 27, Thaw's icon has a macOS menu bar switch separate from Thaw's own icon setting; both must allow the icon to appear. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x44 | behavior | menu bar | On macOS 26, macOS inserts new status items at the far left, which Thaw treats as Hidden or, when enabled, Always Hidden. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x45 | behavior | menu bar | Some apps do not persist their menu bar item position across relaunch, so Thaw may treat the item as new even after it was moved. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x46 | workaround | menu bar | Option-clicking the Thaw icon reveals the always-hidden section; double-clicking an empty menu bar area can do so if enabled in Advanced settings. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x47 | workaround | menu bar | Command-dragging an item in the menu bar moves it to another section. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x48 | behavior | menu bar | Thaw cannot rearrange menu bar items while Automatically hide and show the menu bar is active, because macOS does not expose enough of the menu bar. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x49 | behavior | menu bar | Some apps draw icons outside the status-item APIs Thaw enumerates, or host an icon under com.apple.controlcenter without a stable identifier, so the icon may be visible in the menu bar but absent by name from Menu Bar Layout settings. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x50 | behavior | menu bar | macOS prevents certain system items from being repositioned with Command-drag: they follow the cursor during the drag and snap back on release. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x51 | bug | menu bar | Display changes can cause brief menu bar visual glitches, including resolution flicker and extra spacing; these are often transient. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x52 | behavior | menu bar | When a display transition requires different menu bar spacing, Thaw may relaunch apps with menu bar items; duplicate icons can briefly appear if the host app also relaunches its agent. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x53 | bug | menu bar | Spacing values far from the default, especially negative spacing, can put items under the notch or off-screen even though they remain listed in Layout settings. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x54 | behavior | menu bar | For user apps, Thaw requests that they quit and relaunches them; it does not force-quit them. An app that declines to quit keeps running with its previous spacing. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x55 | workaround | menu bar | Setting When applying spacing to Wait until next restart makes Thaw write the new system preference without restarting apps; the new spacing appears when each app next starts. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x56 | behavior | menu bar | Screen Recording is used for live previews and wallpaper-derived tints. Without permission, Thaw shows each item's owning app icon in the Thaw Bar, layout bars, and search panel. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x57 | behavior | menu bar | Turning off Use Thaw Bar stops Thaw from actively concealing items, but icons hidden while the bar was enabled may remain visible until Thaw manages the layout again. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x58 | bug | menu bar | Live Activities mirrored from an iPhone can interfere with layout operations; gaps or empty spaces in the menu bar during mirroring are a known limitation. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x59 | bug | menu bar | Mission Control and space switches can trigger full layout redraw storms. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.01.04.x60 | new | menu bar | In Beta Four, Bartender can experimentally detect some apps hidden through Control Center settings and display them in a separate Disabled Items section; detection may miss items, especially when the menu bar is overflowing. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x61 | changed | menu bar | In Beta Four, wider items, such as meeting titles and network activity displays, appear at a more natural size in Bartender Bar and the layout editor. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x62 | changed | menu bar | In Beta Four, spacing between items in Bartender Bar was improved to remove oversized gaps around some icons and wider items. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x63 | new | menu bar | In Beta Four, Bartender displays an “Updating your Menu Bar” message and spinner while refreshing or recapturing; consecutive updates keep the message visible. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x64 | changed | menu bar | In Beta Four, the Menu Bar page and layout editor should stutter less while items update. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x65 | changed | menu bar | In Beta Four, hiding or rearranging an item updates the layout sooner. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x66 | fix | menu bar | In Beta Four, temporary image-capture failures should no longer replace a usable menu bar image with the app icon. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x67 | fix | menu bar | In Beta Four, when an item's width changes and its old image no longer fits, Bartender replaces it with a fresh capture when possible. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x68 | fix | menu bar | In Beta Four, Recapture keeps existing usable images when a replacement cannot be captured, avoids repeatedly attempting unsuccessful captures, and yields when the user opens an item. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x69 | behavior | menu bar | In Beta Four, apps with several menu bar items still share visibility: items can be ordered separately, but showing or hiding one affects the others. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x70 | workaround | menu bar | In Beta Four, the native overflow may flicker briefly during some rearrangements; automatically moving items that do not fit into Hidden can help. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x71 | new | menu bar | In Beta Three, menu bar spacing options returned under Customize on the Menu Bar page: Default, Small, Tiny, and No Spacing. Users must sign out and back in for apps to pick up the change. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x72 | new | menu bar | In Beta Three, an option in General temporarily hides normally shown items when hidden items are revealed by clicking the Bartender item or using a hotkey, then restores the usual bar. The clock and Control Center remain visible; the option is unavailable with Bartender Bar. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x73 | new | menu bar | In Beta Three, menu bar styling offers material backgrounds with adjustable thickness. Glass and material styles can use Light, Dark, Match System, or Match Menu Bar; Match Menu Bar follows the appearance of the menu bar icons. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x74 | new | menu bar | In Beta Three, Bartender Bar's glass background offers the same appearance choices as the menu bar styles. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x75 | fix | menu bar | In Beta Three, glass and material menu bar styles retain their highlights when another app is active. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x76 | fix | menu bar | In Beta Three, capsule menu bar styles have rounded ends on both sides, including when using separate pills. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x77 | new | menu bar | In Beta Three, the Menu Bar page has a Menu Bar Access reminder below its header; Grant Access can retry access without returning to setup. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x78 | new | menu bar | In Beta Two, users can add spacers to the menu bar, set their width and optional text, emoji, or SF Symbol, and save them per profile. Spacers are added by dragging the “Drag to add a spacer” chip from the palette beneath Always Hidden into Shown or Hidden. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x79 | new | menu bar | In Beta Two, with Show Bar Item Images enabled, Hidden rows on the Layout page display captured menu bar item images instead of generic app icons. Images refresh at launch and when the option is enabled; Recapture Items refreshes them manually. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x80 | new | menu bar | In Beta Two, Show App Icons overlays each item with its owning app's icon, or the Apple logo for system items. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x81 | changed | menu bar | In Beta Two, Layout options are grouped into a Customize section, including Show App Icons, Show Bar Item Images, and Spacers & Groups. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x82 | fix | menu bar | In Beta Two, moving an app's item into Always Hidden also moves its other items there; an app can no longer be listed in two sections at once, and previously split layouts are corrected. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x83 | behavior | menu bar | In Beta Two, apps with multiple items can more reliably have their items rearranged independently within a section, but showing or hiding one item still affects all items from that app. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x84 | fix | menu bar | In Beta Two, constantly updating item labels should no longer cause items to duplicate, drift between positions, or return to Shown. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x85 | fix | menu bar | In Beta Two, Bartender reduced menu bar flickering and repeated reloads, especially for apps that update constantly. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x86 | new | menu bar | In Beta One, Bartender can hide system items such as the clock or Control Center, but cannot move them within the shown section. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x87 | new | menu bar | In Beta One, presets can be linked directly to monitor setups without an Automation, so a preset can change when displays are connected or disconnected. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x88 | changed | menu bar | In Beta One, presets were moved into the Layout page. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x89 | behavior | menu bar | In Beta One, Bartender supports most macOS 27 overflow behaviors; automatic handling is available to reduce conflicts, although the bar may flicker temporarily while settling on the correct width. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x90 | changed | menu bar | In Golden Gate Preview One, Bartender Bar appears beneath the point where the user clicks or hovers in the menu bar, rather than always at the end. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x91 | behavior | menu bar | In Golden Gate Preview One, Bartender Bar shows at the mouse location when clicked or hovered, and its autohide timer waits while an item's own menu is open. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x92 | behavior | menu bar | In Golden Gate Preview One, Bartender Bar displays app icons instead of actual menu bar item images; clicking an item shows the real icon in the menu bar. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x93 | changed | menu bar | In Golden Gate Preview One, Bartender no longer moves items by interacting with the mouse, and interacts with the mouse only when the user activates an item through Bartender Bar. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x94 | behavior | menu bar | In Golden Gate Preview One, the menu bar background shown in layout pages and Bartender Bar updates less frequently and may not reflect frequently rotated desktop backgrounds. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x95 | known issue | menu bar | In Golden Gate Preview One, the menu bar item may not respond while Bartender indexes the bar; users can right-click it again after indexing finishes. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x96 | known issue | menu bar | In Golden Gate Preview One, menu bar behavior on notched MacBooks is not fully stable, and overflowing items may have incorrect image previews or drift from the live layout. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x97 | workaround | menu bar | In Golden Gate Preview One, after moving enough items, the menu bar may stop responding; the temporary Restart Menu Bar button is a workaround. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x98 | changed | menu bar | In Golden Gate Preview Three, the layout section settles right-to-left, matching the menu bar's direction. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x99 | changed | menu bar | In Golden Gate Preview Three, items shown in Hidden appear only when they are currently running. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.01.04.x100 | behavior | menu bar | The icons on the right side of the menu bar are called status menus and can show status information such as battery percentage and Wi-Fi, as well as provide actions such as turning on Bluetooth. | T4 [Koffret](https://koffret.com/learn/mac-login-screen) |
| MC.01.04.x101 | behavior | menu bar | Menu bar controls are selected in System Settings by clicking Menu Bar in the sidebar and selecting the checkbox next to each control. | T4 [Koffret](https://koffret.com/learn/mac-login-screen) |
| MC.01.04.x102 | behavior | menu bar | Fast User Switching can be added to the menu bar through Menu Bar settings by selecting Fast User Switching and choosing an option from its pop-up menu, or through Add Controls > Fast User Switching > Add to Menu Bar. | T4 [Koffret](https://koffret.com/learn/mac-login-screen) |
| MC.01.04.x103 | behavior | menu bar | Once added to the menu bar, Fast User Switching appears at the top right and can switch accounts; on a Mac with Touch ID, a fingerprint can be used to switch. | T4 [Koffret](https://koffret.com/learn/mac-login-screen) |
| MC.01.04.x104 | behavior | menu bar | On a MacBook with a notch, a long app menu can push right-side menu bar icons out of view. | T4 [Koffret](https://koffret.com/learn/mac-login-screen) |
| MC.01.04.x105 | behavior | menu bar | Full Keyboard Access can move keyboard focus to the menu bar; Fn-Control-F2 jumps there, after which arrow keys or typing can be used to find an item. | T4 [Lets Suite](https://www.lets-suite.com/blog/full-keyboard-access-mac) |
| MC.01.04.x106 | new | menu bar | The post reports a new menu bar battery icon similar to the iOS and iPadOS design. | T5 [MacRumors Forums](https://forums.macrumors.com/threads/macos-27-all-the-little-things.2483520/) |
| MC.01.04.x107 | new | menu bar | The post reports a new Ethernet status indicator in the menu bar. | T5 [MacRumors Forums](https://forums.macrumors.com/threads/macos-27-all-the-little-things.2483520/) |
| MC.01.04.x108 | new | menu bar | The post reports a new option to hide menu bar icons. | T5 [MacRumors Forums](https://forums.macrumors.com/threads/macos-27-all-the-little-things.2483520/) |
| MC.01.04.x109 | new | menu bar | An Automatic Assessment Configuration can now provide controls for the Dock and Menu Bar, among other testing-environment settings. | T5 [MacRumors Forums](https://forums.macrumors.com/threads/macos-27-all-the-little-things.2483520/) |
| MC.01.04.x110 | new | menu bar | The article lists a macOS 27 change that lets menu bar icons be collapsed so they are not cut off by a MacBook’s notch. | T4 [Gizmodo](https://gizmodo.com/im-more-excited-for-the-tiny-changes-in-apples-27-software-updates-than-siri-ai-tbh-2000769815) |
| MC.01.04.x111 | new | menu bar | The Siri AI app can be opened by clicking it in the Menu bar. | T4 [Tom's Guide](https://www.tomsguide.com/computing/macos/macos-golden-gate-review) |
| MC.01.04.x112 | behavior | menu bar | A top comment reports that menu bar items hidden behind the notch now automatically expand. | T4 [9to5Mac](https://9to5mac.com/2026/09/14/macos-27-golden-gate-now-available-here-is-everything-new/) |

### MC.01.05 Live Activities

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).


#### Manage iPhone notifications and Live Activities on Mac

T1 · OS documentation · [Apple Support](https://support.apple.com/en-us/120684) · macOS 27 text present: no

After iPhone Mirroring connects an iPhone to a Mac, notifications and Live Activities from that iPhone automatically appear on the Mac, even when iPhone Mirroring is not in use. The page explains how to choose an iPhone, control notifications and Live Activities on Mac or iPhone, and remove a Mac from the iPhone Mirroring list.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.01.05.e001 | iPhone Mirroring | other | Mac and iPhone | Used to connect an iPhone to a Mac. The page also refers to using it from another Mac. |
| MC.01.05.e002 | iPhone badge | indicator | Over the app icon in an iPhone notification on Mac | Identifies the notification as coming from an iPhone. |
| MC.01.05.e003 | Live Activity icon | icon | Mac menu bar | A small icon shown when menu bar space is limited. |
| MC.01.05.e004 | compact view | view | Mac menu bar | An alternative to a small Live Activity icon, depending on available menu bar space. |
| MC.01.05.e005 | close button | button | Corner of a Live Activity detail view on Mac | Closes a specific Live Activity for the duration of that activity. |
| MC.01.05.e006 | Apple menu  | menu | Mac menu bar | Menu used to open System Settings. |
| MC.01.05.e007 | System Settings | window | Mac | Contains the settings referenced for iPhone selection, notifications, and Live Activities. |
| MC.01.05.e008 | General | view | System Settings sidebar | Contains AirDrop & Continuity on macOS Tahoe 26.4 or later. |
| MC.01.05.e009 | AirDrop & Continuity | view | System Settings > General | Contains the iPhone pop-up menu on macOS Tahoe 26.4 or later. |
| MC.01.05.e010 | iPhone pop-up menu | control | System Settings > General > AirDrop & Continuity on macOS Tahoe 26.4 or later; System Settings > Desktop & Dock on earlier versions | Used to choose the iPhone the Mac uses for mirroring, notifications, and Live Activities. |
| MC.01.05.e011 | Desktop & Dock | view | System Settings sidebar | Contains the iPhone pop-up menu on earlier macOS versions. |
| MC.01.05.e012 | iPhone Widgets | setting | System Settings > Desktop & Dock | The iPhone pop-up menu appears just below this setting on earlier macOS versions. |
| MC.01.05.e013 | Notifications | view | System Settings sidebar; iPhone Settings | Contains controls for iPhone notifications and Live Activities on Mac, and app notification settings on iPhone. |
| MC.01.05.e014 | Allow notifications from iPhone | setting | System Settings > Notifications | Turns iPhone notifications on or off for the Mac. When on, notification settings can also be controlled for each iPhone app individually. |
| MC.01.05.e015 | Allow Live Activities from iPhone | setting | System Settings > Notifications | Turns Live Activities from iPhone on or off on macOS Tahoe 26 or later. |
| MC.01.05.e016 | Show on Mac | setting | iPhone Settings > Notifications > an app | Controls whether that app's notifications are shown on the Mac. |
| MC.01.05.e017 | Allow Notifications | setting | iPhone Settings > Notifications > an app | Controls whether the app sends notifications to both iPhone and Mac. |
| MC.01.05.e018 | Settings | window | iPhone | Contains the notification, Live Activities, AirPlay & Continuity, and iPhone Mirroring settings referenced. |
| MC.01.05.e019 | Apps | view | iPhone Settings | Contains app-specific settings, including settings for apps that support Live Activities. |
| MC.01.05.e020 | Live Activities | setting | iPhone Settings > Apps > an app that supports Live Activities | Turns Live Activities off or on for the app on iPhone and Mac. |
| MC.01.05.e021 | AirPlay & Continuity | view | iPhone Settings > General | Contains iPhone Mirroring, where the iPhone lists Macs that have used iPhone Mirroring. |
| MC.01.05.e022 | iPhone Mirroring | view | iPhone Settings > General > AirPlay & Continuity | Shows Macs that have used iPhone Mirroring with the iPhone and can receive its notifications and Live Activities. |
| MC.01.05.e023 | Delete | menu item | After swiping left on a Mac name in iPhone Settings > General > AirPlay & Continuity > iPhone Mirroring | Removes that Mac from the iPhone Mirroring list. |
| MC.01.05.e024 | iPhone notification | other | Mac | A notification from an iPhone app, marked with an iPhone badge. |
| MC.01.05.e025 | Live Activity | view | Mac menu bar | An activity from an iPhone that can be opened for more detail. |
| MC.01.05.e026 | related app | other | iPhone Mirroring | The app opened when an iPhone notification is interacted with while the iPhone is nearby. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.01.05.b001 | iPhone Mirroring connects an iPhone to a Mac | The Mac automatically receives notifications and Live Activities from the iPhone, even when iPhone Mirroring is not in use. | The iPhone must be turned on but does not need to be nearby. |
| MC.01.05.b002 | iPhone Mirroring is used to connect the iPhone from another Mac | That Mac also receives notifications and Live Activities from the iPhone. | The other Mac must be used to connect to the iPhone with iPhone Mirroring. |
| MC.01.05.b003 | An iPhone notification appears on Mac | An iPhone badge appears over the app icon in the notification. | The notification is from the iPhone. |
| MC.01.05.b004 | A notification is cleared on iPhone | That iPhone notification is also cleared on Mac. | Applies to the iPhone notification. |
| MC.01.05.b005 | A notification from an app installed on Mac is cleared | It can be cleared only from Mac. | The Mac app's notifications are not iPhone notifications. |
| MC.01.05.b006 | The user interacts with an iPhone notification on Mac | The related app opens in iPhone Mirroring so the user can continue the task there. | The iPhone is nearby. |
| MC.01.05.b007 | An iPhone Live Activity appears on Mac | It appears in the menu bar as either a small Live Activity icon or a compact view. | The form depends on the space available in the menu bar. |
| MC.01.05.b008 | The user clicks a Live Activity in the menu bar | More detail about the activity appears. | The Live Activity is visible in the menu bar. |
| MC.01.05.b009 | The user double-clicks a Live Activity on Mac | iPhone Mirroring opens so the user can continue the task there. | The iPhone is nearby. |
| MC.01.05.b010 | The user chooses an iPhone from the iPhone pop-up menu | The Mac uses that iPhone for mirroring, iPhone notifications, and Live Activities. | More than one iPhone signed in to the same Apple Account must be nearby. |
| MC.01.05.b011 | The Mac detects more than one iPhone that can be used for mirroring | The iPhone pop-up menu appears and a different iPhone can be chosen. | The menu appears only when more than one eligible iPhone is detected. |
| MC.01.05.b012 | The user turns off or on Allow notifications from iPhone | The Mac stops or resumes receiving iPhone notifications. | The setting is in System Settings > Notifications. |
| MC.01.05.b013 | Allow notifications from iPhone is on | The user can also control notifications for each iPhone app individually. | The setting is in System Settings > Notifications. |
| MC.01.05.b014 | An app's setting says mirroring is disabled from the iPhone | Open Settings on iPhone, tap Notifications, and check that app's settings, including Show on Mac. | Applies to the indicated app. |
| MC.01.05.b015 | An app's setting says the app is installed on this Mac | Notifications can be received only from the Mac app, not also from the iPhone version of the app. | Applies to that app. |
| MC.01.05.b016 | The user turns off or on Allow Live Activities from iPhone | The Mac stops or resumes receiving Live Activities from iPhone. | Available in macOS Tahoe 26 or later; the setting is in System Settings > Notifications. |
| MC.01.05.b017 | The user moves the mouse over a Live Activity and clicks the close button | That specific Live Activity is turned off for the duration of the activity. | The Live Activity must appear in the menu bar; click it for more detail first. |
| MC.01.05.b018 | The user turns off Allow Notifications for an app on iPhone | Both iPhone and Mac stop receiving notifications from that app. | The setting is in iPhone Settings > Notifications > the app. |
| MC.01.05.b019 | The user turns off Show on Mac for an app on iPhone | Only the Mac stops receiving notifications from that app. | The setting is in iPhone Settings > Notifications > the app. |
| MC.01.05.b020 | The user turns Live Activities off or on for an app on iPhone | Live Activities for that app are turned off or on for both iPhone and Mac. | The app must support Live Activities; the setting is in Settings > Apps > the app > Live Activities. |
| MC.01.05.b021 | The user swipes left on a Mac name and taps Delete | That Mac can no longer receive notifications or Live Activities from the iPhone. | The Mac can receive them again if it is used to set up iPhone Mirroring with the iPhone again. |
| MC.01.05.b022 | The Mac is set up to make and receive calls from the iPhone | The Mac always receives notifications about incoming calls from the iPhone. | This is not related to iPhone Mirroring. |
| MC.01.05.b023 | Screen Sharing notifications are changed on iPhone | This does not affect iPhone Mirroring notifications. | The Allow Notifications setting in Screen Sharing settings concerns notifications when using SharePlay or Screen Mirroring. |
| MC.01.05.b024 | The user looks for the iPhone selection menu on earlier macOS versions | The menu is below the iPhone Widgets setting in Desktop & Dock. | The menu appears only when the Mac detects more than one iPhone that can be used for mirroring. |
| MC.01.05.b025 | The user looks for a setting that appears only after iPhone Mirroring has been used successfully | Set up and use iPhone Mirroring with the device, then reopen Settings and check again. | Applies when the referenced setting is not visible. |

System Settings paths referenced: Mac: Apple menu  > System Settings > General > AirDrop & Continuity > iPhone pop-up menu (macOS Tahoe 26.4 or later); Mac: Apple menu  > System Settings > Desktop & Dock > iPhone pop-up menu (earlier macOS versions; the menu is just below iPhone Widgets); Mac: Apple menu  > System Settings > Notifications > Allow notifications from iPhone; Mac: Apple menu  > System Settings > Notifications > Allow Live Activities from iPhone; iPhone: Settings > Notifications > an app > Allow Notifications; iPhone: Settings > Notifications > an app > Show on Mac; iPhone: Settings > Apps > an app that supports Live Activities > Live Activities; iPhone: Settings > General > AirPlay & Continuity > iPhone Mirroring

#### Use Continuity to work across Apple devices

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchl1d734309/27/mac/27) · macOS 27 text present: no

Apple Support describes Continuity features for working across a Mac, iPhone, iPad, and Apple Watch, plus other ways to connect or sync devices.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.01.05.e027 | Quickly share content between devices | view | Section heading | Section about sharing content with nearby Apple devices. |
| MC.01.05.e028 | Use AirDrop to send items to nearby Apple devices | other | Below the AirDrop description | Link about sending items with AirDrop. |
| MC.01.05.e029 | Use your iPhone camera with your Mac | view | Section heading | Section about Continuity Camera. |
| MC.01.05.e030 | Use your iPhone as a webcam on Mac | other | Below the Continuity Camera description | Link about using an iPhone as a Mac webcam. |
| MC.01.05.e031 | Control your iPhone from your Mac | view | Section heading | Section about controlling an iPhone from a Mac. |
| MC.01.05.e032 | Control your iPhone from your Mac | other | Below the section description | Link about controlling an iPhone from a Mac. |
| MC.01.05.e033 | Extend your workspace with an iPad | view | Section heading | Section about Sidecar. |
| MC.01.05.e034 | Use your iPad as a second display for your Mac | other | Below the Sidecar description | Link about using an iPad as a second display. |
| MC.01.05.e035 | Even more Continuity features | view | Section heading | Section introducing additional Continuity features. |
| MC.01.05.e036 | Copy and paste | other | Continuity features table | Feature label for Universal Clipboard. |
| MC.01.05.e037 | Copy and paste between your Mac and other Apple devices | other | Continuity features table | Link about Universal Clipboard. |
| MC.01.05.e038 | Hand off tasks | other | Continuity features table | Feature label for Handoff. |
| MC.01.05.e039 | Hand off tasks between Apple devices | other | Continuity features table | Link about Handoff. |
| MC.01.05.e040 | Control your Mac and iPad together | other | Continuity features table | Feature label for Universal Control. |
| MC.01.05.e041 | Use one keyboard and mouse to control Mac and iPad | other | Continuity features table | Link about Universal Control. |
| MC.01.05.e042 | Stream to a larger screen | other | Continuity features table | Feature label for streaming with AirPlay. |
| MC.01.05.e043 | Stream video and audio with AirPlay | other | Continuity features table | Link about AirPlay. |
| MC.01.05.e044 | Answer calls and send texts from your Mac | other | Continuity features table | Feature label for calls and texts on Mac. |
| MC.01.05.e045 | Receive phone calls and text messages on your Mac | other | Continuity features table | Link about calls and texts on Mac. |
| MC.01.05.e046 | Share your Wi-Fi | other | Continuity features table | Feature label for using an iPhone or iPad Personal Hotspot. |
| MC.01.05.e047 | Share the internet connection from your iPhone or iPad with your Mac | other | Continuity features table | Link about sharing an internet connection. |
| MC.01.05.e048 | Insert sketches, photos, and scans | other | Continuity features table | Feature label for inserting content from an iPhone or iPad. |
| MC.01.05.e049 | Insert sketches, photos, and scans from iPhone or iPad on Mac | other | Continuity features table | Link about inserting content on Mac. |
| MC.01.05.e050 | Unlock your Mac with Apple Watch | other | Continuity features table | Feature label for unlocking and approving requests. |
| MC.01.05.e051 | Unlock your Mac and approve requests with Apple Watch | other | Continuity features table | Link about Apple Watch authentication. |
| MC.01.05.e052 | Other ways to use your devices together | view | Section heading | Section listing additional ways to use a Mac with other Apple devices. |
| MC.01.05.e053 | Continuity features and requirements for Apple devices | other | Note below the introduction | Link to information about minimum system requirements and feature requirements. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.01.05.b026 | Sign in to the same Apple Account on each device. | You can get started using Continuity features; no additional app needs to be installed. | The devices are a Mac and another Apple device, such as an iPhone, iPad, or Apple Watch. |
| MC.01.05.b027 | Use AirDrop to share items. | Photos, videos, contacts, and more can be shared between nearby Apple devices, including iPhone, iPad, and Mac. | The devices are nearby; the article describes sharing as simple as dragging and dropping. |
| MC.01.05.b028 | Use an iPhone with a Mac through Continuity Camera. | The iPhone can serve as the Mac's webcam or microphone; video effects such as Desk view can also be used when it serves as a webcam. | An iPhone and Mac are used together. |
| MC.01.05.b029 | Control an iPhone from a Mac. | You can control the iPhone and receive iPhone notifications and Live Activities on the Mac. | An iPhone and Mac are used together. |
| MC.01.05.b030 | Use Sidecar with an iPad and Mac. | The iPad becomes a second display for the Mac. You can extend the Mac desktop, or show the same app on both screens while controlling different aspects of it. | An iPad and Mac are used together. |
| MC.01.05.b031 | Copy content on one Apple device and paste it on another. | Universal Clipboard lets you copy and paste text, images, photos, and videos across devices. | The devices are Apple devices. |
| MC.01.05.b032 | Start a task on one device and continue on another. | Handoff lets you pick up where you left off. | The task can include creating a document, writing an email, or browsing the web. |
| MC.01.05.b033 | Use Universal Control. | A single keyboard and trackpad or mouse can be used to work across an iPad and Mac. | An iPad and Mac are used together. |
| MC.01.05.b034 | Stream content with AirPlay. | Videos, music, photos, and more can be shared from an iPhone or iPad to a Mac, or from a Mac to a TV. | The listed source and destination devices are used. |
| MC.01.05.b035 | Use an iPhone with a Mac for calls and texts. | Calls can be relayed from the iPhone, and texts can be sent and received on the Mac. | An iPhone and Mac are used together. |
| MC.01.05.b036 | Connect a Mac to an iPhone or iPad Personal Hotspot. | The Mac can use the device's internet connection. | The iPhone or iPad has Wi-Fi + Cellular. |
| MC.01.05.b037 | Take a picture or scan a document with an iPhone or iPad camera. | The picture or scan appears instantly on the Mac. | An iPhone or iPad camera and a Mac are used together. |
| MC.01.05.b038 | Use an Apple Watch with a Mac. | The Apple Watch can unlock the Mac or approve authentication requests. | An Apple Watch and Mac are used together. |
| MC.01.05.b039 | Use Continuity features. | Some features require Wi-Fi and Bluetooth. | The article says minimum system requirements apply; it does not specify them. |
| MC.01.05.b040 | Connect an iPhone and a computer with a cable. | The devices can be connected by cable. | No additional conditions stated. |
| MC.01.05.b041 | Use iCloud across devices. | Information can be kept up to date across all your devices. | No additional conditions stated. |
| MC.01.05.b042 | Sync content between a Mac and an iPhone, iPad, or iPod touch with a cable. | Content can be synced between the devices. | A cable is used. |
| MC.01.05.b043 | Connect external displays to a Mac. | One or more external displays can be connected. | No additional conditions stated. |
| MC.01.05.b044 | Connect storage devices to a Mac. | The Mac can use storage devices, such as an external hard drive. | No additional conditions stated. |
| MC.01.05.b045 | Use iPhone Mirroring. | The feature is available only where it is offered; it is currently unavailable in the European Union. | It must be available in your country or region. |

#### Control your iPhone from your Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchl444d53a6/27/mac/27) · macOS 27 text present: no

The article explains how to set up and use iPhone Mirroring on a Mac, including connecting to an iPhone, interacting with its screen, receiving notifications and Live Activities, transferring files, and troubleshooting.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.01.05.e054 | iPhone Mirroring | app | Mac | App used to connect to and control an iPhone. |
| MC.01.05.e055 | Continue | button | iPhone Mirroring setup | Continues setup. |
| MC.01.05.e056 | Ask Every Time | setting | iPhone Mirroring setup | Authentication choice that prompts for Touch ID, Face ID, or the iPhone passcode each time iPhone Mirroring is used. |
| MC.01.05.e057 | Authenticate Automatically | setting | iPhone Mirroring setup | Authentication choice that connects to iPhone Mirroring without requiring authentication. |
| MC.01.05.e058 | Trust This Computer? | alert | iPhone | Alert that may appear during setup; the article says to tap Trust. |
| MC.01.05.e059 | Trust | button | Trust This Computer? alert on iPhone | Allows the computer to be trusted. |
| MC.01.05.e060 | Allow accessory to connect? | alert | Mac | Alert that may appear during setup; the article says to click Allow. |
| MC.01.05.e061 | Allow | button | Allow accessory to connect? alert on Mac | Allows the accessory to connect. |
| MC.01.05.e062 | Allow Notifications from iPhone | setting | iPhone Mirroring setup | Choice to allow iPhone notifications on the Mac. |
| MC.01.05.e063 | iPhone Mirroring Settings | window | iPhone Mirroring app on Mac | Settings area used to change the iPhone selected for mirroring. |
| MC.01.05.e064 | Change iPhone | button | iPhone Mirroring Settings | Opens the controls for choosing which iPhone to use. |
| MC.01.05.e065 | Widgets | section | iPhone Mirroring Settings | Section containing the iPhone pop-up menu. |
| MC.01.05.e066 | iPhone | pop-up menu | Under Widgets in iPhone Mirroring Settings | Menu for choosing an iPhone. |
| MC.01.05.e067 | System Settings | app | Mac | The article says it can also be opened when choosing an iPhone, but does not provide a path within the app. |
| MC.01.05.e068 | View | menu | iPhone Mirroring menu bar | Contains window-size options. |
| MC.01.05.e069 | iPhone Size | menu item | View menu | Returns the window to its original iPhone size. |
| MC.01.05.e070 | Actual Size | menu item | View menu | Shows the original content without scaling. |
| MC.01.05.e071 | Zoom In | menu item | View menu | Shows the same content scaled larger. |
| MC.01.05.e072 | Zoom Out | menu item | View menu | Shows the same content scaled smaller. |
| MC.01.05.e073 | Edit Home Screen | menu item | Control-click menu for an app on the iPhone Home Screen | Enters Home Screen editing. |
| MC.01.05.e074 | Live Activity | indicator | Mac menu bar | Shows an iPhone Live Activity; clicking it displays more information. |
| MC.01.05.e075 | Settings | app | iPhone | The article directs users to this app for notification settings and for turning off iPhone Mirroring. |
| MC.01.05.e076 | General | setting | iPhone Settings | Path component for iPhone Mirroring settings. |
| MC.01.05.e077 | AirPlay & Continuity | setting | iPhone Settings > General | Path component for iPhone Mirroring settings. |
| MC.01.05.e078 | iPhone Mirroring | setting | iPhone Settings > General > AirPlay & Continuity | Settings page used to remove a Mac from iPhone Mirroring. |
| MC.01.05.e079 | Edit | button | Top right of iPhone Mirroring settings on iPhone | Used before selecting a Mac to remove. |
| MC.01.05.e080 | Handoff | setting | iPhone and Mac | The article says it should be turned on when troubleshooting iPhone Mirroring. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.01.05.b046 | Before using iPhone Mirroring | Both devices must have Wi-Fi and Bluetooth turned on, use the same Apple Account, be within 30 feet (10 meters), and meet minimum system requirements. The Mac must have macOS 15 or later and the iPhone must have iOS 18 or later. | These are stated prerequisites for iPhone Mirroring. |
| MC.01.05.b047 | Using a VPN while relying on Continuity features | Ensure the VPN configuration does not prevent local networking, because that may interfere with some Continuity features. | Applies when a VPN is in use. |
| MC.01.05.b048 | Attempting to use iPhone Mirroring in a country or region where it is unavailable | iPhone Mirroring says it is unable to connect to the iPhone for that reason. | The article says iPhone Mirroring is not currently available in the European Union. |
| MC.01.05.b049 | Starting setup | Make sure the iPhone is turned on, nearby, and locked; open iPhone Mirroring on the Mac and click Continue. | The stated setup steps. |
| MC.01.05.b050 | Continuing iPhone Mirroring setup | Enter the iPhone passcode and choose an authentication option if prompted. | The available choices described are Ask Every Time and Authenticate Automatically. |
| MC.01.05.b051 | Choosing Ask Every Time | A prompt for Touch ID, Face ID, or the iPhone passcode appears each time iPhone Mirroring is used. | During setup. |
| MC.01.05.b052 | Choosing Authenticate Automatically | iPhone Mirroring connects without requiring authentication. | During setup. |
| MC.01.05.b053 | The iPhone displays the Trust This Computer? alert | Tap Trust; the iPhone may then ask for its passcode. | May occur during setup. |
| MC.01.05.b054 | The Mac displays the “Allow accessory to connect?” alert | Click Allow. | May occur during setup. |
| MC.01.05.b055 | Completing setup | Choose whether to allow notifications from the iPhone. | During iPhone Mirroring setup. |
| MC.01.05.b056 | Setting up more than one iPhone for a Mac | You can set up multiple iPhones but connect to only one at a time. | For iPhone Mirroring on a Mac. |
| MC.01.05.b057 | Choosing a different iPhone | In iPhone Mirroring Settings, click Change iPhone, then use the iPhone pop-up menu under Widgets to choose an iPhone. | On the Mac. |
| MC.01.05.b058 | Clicking in the iPhone Mirroring app | The click acts as a tap on the iPhone. | When using iPhone Mirroring. |
| MC.01.05.b059 | Swiping or scrolling in the iPhone Mirroring app | The corresponding swipe or scroll is performed on the mirrored iPhone. | When using iPhone Mirroring. |
| MC.01.05.b060 | Typing with the keyboard in the iPhone Mirroring app | Text is entered on the iPhone. | When using iPhone Mirroring. |
| MC.01.05.b061 | Moving the pointer to the top of the iPhone Mirroring window and dragging the title bar | The window is repositioned. | The title bar appears when the pointer is moved to the top of the window. |
| MC.01.05.b062 | Holding the pointer near the edge of the iPhone Mirroring window until it becomes a double arrow, then dragging | The window is resized when resizing is supported. On a resizable app, the amount of content shown changes; on the Home Screen, App Switcher, or a non-resizable app, content scales larger or smaller. | Window resizing requires iOS 27 or later and is not supported on iPhone 11, 12, or 13 (any model), or iPhone SE (2nd or 3rd generation). On unsupported models, the resize cursor scales the window instead of resizing it. |
| MC.01.05.b063 | Resizing an app in iPhone Mirroring | The app reopens at that size the next time it is used in iPhone Mirroring. | Choose View > iPhone Size to return it to its original size. |
| MC.01.05.b064 | Choosing View > iPhone Size | The window returns to its original iPhone size. | In the iPhone Mirroring menu bar. |
| MC.01.05.b065 | Choosing View > Actual Size | The original content is shown without scaling. | In the iPhone Mirroring menu bar. |
| MC.01.05.b066 | Choosing View > Zoom In | The same content is shown scaled larger. | In the iPhone Mirroring menu bar. |
| MC.01.05.b067 | Choosing View > Zoom Out | The same content is shown scaled smaller. | In the iPhone Mirroring menu bar. |
| MC.01.05.b068 | Moving the pointer to the top of the iPhone Mirroring app and clicking the App Switcher control | The App Switcher opens. | The title bar appears when the pointer is moved to the top; the article does not provide a text label for the control. |
| MC.01.05.b069 | While in an app, moving the pointer to the top of iPhone Mirroring and clicking the Home Screen control | The Home Screen opens and the window returns to iPhone size. | The article does not provide a text label for the control. |
| MC.01.05.b070 | Control-clicking an app on the Home Screen and clicking Edit Home Screen | The Home Screen enters editing mode. | In iPhone Mirroring. |
| MC.01.05.b071 | Control-clicking an app on the Home Screen | You can choose to turn some apps into widgets or perform quick actions, such as creating a new note in Notes or sending your location in Maps. | In iPhone Mirroring; the article does not specify which apps support each option. |
| MC.01.05.b072 | Opening an iPhone widget on the Mac | The widget opens in the iPhone Mirroring app. | When using iPhone Mirroring. |
| MC.01.05.b073 | Setting up iPhone Mirroring with iPhone notifications enabled | iPhone notifications appear on the Mac with an iPhone badge. | The iPhone does not need to be nearby for its notifications to appear, but it must be turned on. |
| MC.01.05.b074 | Changing notification choices after setup | You can turn on iPhone notifications or choose which apps send notifications to the Mac. | The article directs users to Settings on iPhone and Apple menu on Mac but does not provide the remaining settings paths. |
| MC.01.05.b075 | Setting up iPhone Mirroring | Live Activities from the iPhone appear in the Mac menu bar. | When Live Activities are available from the iPhone. |
| MC.01.05.b076 | Clicking a Live Activity in the Mac menu bar | More information about the Live Activity is shown. | On the Mac. |
| MC.01.05.b077 | Double-clicking a Live Activity | The relevant app opens in iPhone Mirroring on the Mac. | On the Mac. |
| MC.01.05.b078 | Clicking outside a Live Activity | The Live Activity closes, but remains visible in the menu bar. | When viewing a Live Activity on the Mac. |
| MC.01.05.b079 | Moving the pointer over a Live Activity and clicking | The article describes this interaction but omits the target and resulting action. | The control or icon to click is not shown in the supplied text. |
| MC.01.05.b080 | Dragging and dropping a file, photo, video, or other item from the Mac to the iPhone Mirroring window, or vice versa | The item is transferred between the Mac and iPhone. | Only in supported apps. |
| MC.01.05.b081 | Removing a Mac from iPhone Mirroring | On iPhone, go to Settings > General > AirPlay & Continuity > iPhone Mirroring, tap Edit in the top right, then select a Mac to remove. | To turn off iPhone Mirroring for that Mac. |
| MC.01.05.b082 | Troubleshooting an iPhone Mirroring connection | Check that both devices use the same Apple Account; Wi-Fi, Bluetooth, and Handoff are turned on; and the devices are within 30 feet (10 meters). | If iPhone Mirroring is not working. |
| MC.01.05.b083 | Having more than one iPhone set up when troubleshooting | On the Mac, open iPhone Mirroring Settings, click Change iPhone, and choose the iPhone to use. | If the wrong iPhone is selected. |
| MC.01.05.b084 | Using the camera or microphone while iPhone Mirroring is active | Those iPhone features are not compatible with iPhone Mirroring. | While using iPhone Mirroring. |
| MC.01.05.b085 | Using Continuity Camera or AirPlay on the Mac while iPhone Mirroring is active | Those Continuity features are not available on the Mac. | While using iPhone Mirroring. |

System Settings paths referenced: iPhone Settings > General > AirPlay & Continuity > iPhone Mirroring; Settings on iPhone — path not specified; Apple menu on Mac — path not specified; System Settings on Mac — no specific path provided

#### iPhone Mirroring: Use your iPhone from your Mac

T1 · OS documentation · [Apple Support](https://support.apple.com/en-us/120421) · macOS 27 text present: yes

Apple Support instructions for setting up and using iPhone Mirroring, including interaction, window sizing, notifications, copying items, settings, device selection, and troubleshooting.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.01.05.e081 | iPhone Mirroring | app | Mac Dock, Applications folder, Spotlight, Siri, and menu bar | The Mac app used to connect to and interact with an iPhone. |
| MC.01.05.e082 | iPhone Mirroring window | window | Mac | Displays the iPhone screen; its toolbar appears when the pointer is moved to the top. |
| MC.01.05.e083 | Home Screen button | button | Toolbar at the top of the iPhone Mirroring window | Opens the iPhone Home Screen. |
| MC.01.05.e084 | App Switcher button | button | Toolbar at the top of the iPhone Mirroring window | Opens the iPhone App Switcher. |
| MC.01.05.e085 | iPhone size button | button | iPhone Mirroring toolbar | Returns the window to standard iPhone size in macOS 27 or later. |
| MC.01.05.e086 | Home Screen | view | iPhone Mirroring | The iPhone Home Screen; its Search field can be clicked to open Spotlight. |
| MC.01.05.e087 | Search field | control | Bottom of the Home Screen in iPhone Mirroring | Opens Spotlight when clicked. |
| MC.01.05.e088 | View | menu | Mac menu bar | Provides the Larger, Smaller, and Actual Size window size choices. |
| MC.01.05.e089 | Larger | menu item | View menu | Selects a larger preset window size. |
| MC.01.05.e090 | Smaller | menu item | View menu | Selects a smaller preset window size. |
| MC.01.05.e091 | Actual Size | menu item | View menu | Selects the actual-size preset. |
| MC.01.05.e092 | Allow | button | Mac prompt about iPhone notifications | Allows notifications from the iPhone. |
| MC.01.05.e093 | Don’t Allow | button | Mac prompt about iPhone notifications | Does not allow notifications from the iPhone. |
| MC.01.05.e094 | Ask every time | setting | iPhone Mirroring settings | Requires authentication before each mirroring session. |
| MC.01.05.e095 | Authenticate automatically | setting | iPhone Mirroring settings | After Mac login authentication to select this setting, does not ask for authentication before mirroring. |
| MC.01.05.e096 | Revoke iPhone Access | setting | iPhone Mirroring settings | Resets setup for this iPhone and removes “Allow notifications from iPhone” from Mac Notifications settings. |
| MC.01.05.e097 | Allow notifications from iPhone | setting | Mac Notifications settings | The notification setting removed when iPhone access is revoked. |
| MC.01.05.e098 | Apple menu | menu | Mac menu bar | Used to open System Settings. |
| MC.01.05.e099 | System Settings | window | Mac | Contains settings referenced for iPhone Mirroring, continuity, notifications, Spotlight, and Lock Screen. |
| MC.01.05.e100 | General | view | System Settings sidebar | Contains AirDrop & Continuity in macOS Tahoe 26.4 or later. |
| MC.01.05.e101 | AirDrop & Continuity | view | System Settings > General | In macOS Tahoe 26.4 or later, contains the iPhone pop-up menu for choosing an iPhone. |
| MC.01.05.e102 | iPhone pop-up menu | control | System Settings > General > AirDrop & Continuity in macOS Tahoe 26.4 or later; System Settings > Desktop & Dock in earlier macOS versions | Chooses which eligible nearby iPhone the Mac uses for mirroring, notifications, and Live Activities. |
| MC.01.05.e103 | Desktop & Dock | view | System Settings sidebar in earlier macOS versions | Contains the iPhone pop-up menu below the iPhone Widgets setting. |
| MC.01.05.e104 | iPhone Widgets | setting | System Settings > Desktop & Dock in earlier macOS versions | The iPhone pop-up menu appears just below this setting. |
| MC.01.05.e105 | Spotlight | view | System Settings sidebar | Contains the iPhone Apps setting. |
| MC.01.05.e106 | iPhone Apps | setting | System Settings > Spotlight | Enables Spotlight or Siri to find apps installed on the iPhone and open them in iPhone Mirroring; requires macOS Tahoe 26 or later. |
| MC.01.05.e107 | Lock Screen | view | System Settings | Contains the “Require password after screen saver begins or display is turned off” setting. |
| MC.01.05.e108 | Require password after screen saver begins or display is turned off | setting | System Settings > Lock Screen | Remains set to Immediately while Authenticate automatically is selected. |
| MC.01.05.e109 | Notifications | view | System Settings on Mac | Contains the “Allow notifications from iPhone” setting. |
| MC.01.05.e110 | AirPlay & Continuity | view | iPhone Settings > General | Contains iPhone Mirroring information identifying the Mac that used the iPhone, including when and for how long. |
| MC.01.05.e111 | iPhone Mirroring | view | iPhone Settings > General > AirPlay & Continuity | Identifies the Mac that used the iPhone, including when and for how long. |
| MC.01.05.e112 | Bluetooth | setting | Mac and iPhone | Must be turned on for iPhone Mirroring. |
| MC.01.05.e113 | Wi-Fi | setting | Mac and iPhone | Must be turned on for iPhone Mirroring. |
| MC.01.05.e114 | Passcode | setting | iPhone | The iPhone must be set up to use a passcode. |
| MC.01.05.e115 | Two-factor authentication | setting | Apple Account on Mac and iPhone | Both devices must be signed in to the same Apple Account using two-factor authentication. |
| MC.01.05.e116 | Wi-Fi Calling | setting | Mac and iPhone | Must be turned on on both devices to make calls from the Mac while iPhone Mirroring is in use. |
| MC.01.05.e117 | iPhone camera and mic | other | iPhone Mirroring | Access is unavailable, including for Face ID and phone calls. |
| MC.01.05.e118 | iPhone Mirroring menu | menu | Mac menu bar | Contains Settings. |
| MC.01.05.e119 | Settings | menu item | iPhone Mirroring menu in the Mac menu bar | Opens iPhone Mirroring settings. |
| MC.01.05.e120 | Photos | app | iPhone | An example source for dragging a video to the Mac desktop. |
| MC.01.05.e121 | Safari | app | Mac | An example source for dragging a photo into an iPhone conversation. |
| MC.01.05.e122 | Messages | app | iPhone | An example destination for a photo dragged from Safari on Mac. |
| MC.01.05.e123 | Mail | app | iPhone | An example destination for a photo dragged from Safari on Mac; also listed as an app designed to show more information when the window is made wider. |
| MC.01.05.e124 | Calendar | app | iPhone | Listed as an app designed to show more information when the window is made wider. |
| MC.01.05.e125 | Files | app | iPhone | Can be used as a source or destination when dragging files between devices. |
| MC.01.05.e126 | iCloud Drive | view | Mac | Can be used as a source or destination when dragging files between devices. |
| MC.01.05.e127 | AirDrop | other | Mac and iPhone | A method for copying items between devices. |
| MC.01.05.e128 | Universal Clipboard | other | Mac and iPhone | A method for copying items between devices. |
| MC.01.05.e129 | Control Center | view | iPhone Mirroring | Can be opened with Command-4 on macOS 27 or later. |
| MC.01.05.e130 | Spotlight | view | iPhone Mirroring | Can be opened from the Home Screen Search field or with Command-3. |
| MC.01.05.e131 | iPhone | device | Near the Mac | Must be locked, turned on, nearby, signed in to the same Apple Account, and set up with a passcode; it can be charging and using StandBy. |
| MC.01.05.e132 | Mac | device | Near the iPhone | Must meet the supported hardware and software requirements and be signed in to the same Apple Account as the iPhone. |
| MC.01.05.e133 | StandBy | view | iPhone | The iPhone can be charging and using StandBy while meeting the proximity requirement. |
| MC.01.05.e134 | AirPlay | other | Mac | The Mac cannot be using AirPlay while meeting the iPhone Mirroring system requirements. |
| MC.01.05.e135 | Sidecar | other | Mac | The Mac cannot be using Sidecar while meeting the iPhone Mirroring system requirements. |
| MC.01.05.e136 | European Union | other | Availability | iPhone Mirroring is currently unavailable in the European Union. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.01.05.b086 | Use iPhone Mirroring | Interact wirelessly with the iPhone and its apps and notifications from the Mac while the iPhone stays locked. | Requires one Mac and one iPhone at a time, meeting the stated system requirements; unavailable in the European Union. |
| MC.01.05.b087 | Meet the Mac system requirements | The Mac can be used with iPhone Mirroring. | Mac uses macOS Sequoia 15 or later and has Apple silicon or the Apple T2 Security Chip. Live Activities on Mac requires macOS Tahoe 26 or later. |
| MC.01.05.b088 | Meet the iPhone system requirements | The iPhone can be used with iPhone Mirroring. | iPhone uses iOS 18 or later and is set up to use a passcode. |
| MC.01.05.b089 | Prepare both devices for iPhone Mirroring | The devices meet the stated account and connectivity requirements. | Both use the same Apple Account with two-factor authentication; Bluetooth and Wi-Fi are on; the iPhone is locked and near the Mac; the Mac is not sharing its internet connection or using AirPlay or Sidecar. |
| MC.01.05.b090 | Click iPhone Mirroring in the Dock, or open it from Applications, Spotlight, or Siri | Start iPhone Mirroring. | The app is included with macOS Sequoia 15 or later. |
| MC.01.05.b091 | The Mac asks the user to unlock the iPhone | Enter the iPhone passcode on the iPhone. | During setup or connection. |
| MC.01.05.b092 | The Mac asks whether to allow iPhone notifications | Choose Allow or Don’t Allow. | The choice can later be changed in Notifications settings on the Mac. |
| MC.01.05.b093 | The Mac asks whether to require Mac login to access the iPhone | Choose whether authentication is required every time or happens automatically. | The choice can later be changed in iPhone Mirroring settings. |
| MC.01.05.b094 | iPhone Mirroring displays the iPhone screen | The Mac is connected to the iPhone and can interact with it. | After connection. |
| MC.01.05.b095 | Quit iPhone Mirroring or unlock the iPhone | Stop mirroring the iPhone. | Either action stops mirroring. |
| MC.01.05.b096 | Remain inactive for a period | iPhone Mirroring automatically pauses the connection. | While mirroring. |
| MC.01.05.b097 | Use the iPhone while mirroring | The iPhone indicates that it is in use and identifies the Mac using it. | While mirroring. |
| MC.01.05.b098 | Unlock the iPhone after mirroring | The iPhone indicates it was used from a Mac and links to iPhone Settings > General > AirPlay & Continuity > iPhone Mirroring, where the Mac, when, and duration are identified. | After mirroring. |
| MC.01.05.b099 | Click or click and hold in the mirrored iPhone screen | Use the Mac mouse or trackpad to perform the corresponding iPhone tap or touch-and-hold action, including editing the Home Screen or opening a quick actions menu. | While interacting with iPhone Mirroring. |
| MC.01.05.b100 | Scroll or swipe with the Mac mouse or trackpad | Scroll or swipe on the iPhone. | While interacting with iPhone Mirroring. |
| MC.01.05.b101 | Press and hold Shift while scrolling | Swipe left or right. | While interacting with iPhone Mirroring. |
| MC.01.05.b102 | Type using the Mac keyboard | Enter text on the iPhone and use keyboard shortcuts. | While interacting with iPhone Mirroring. |
| MC.01.05.b103 | Use a wireless game controller with a supported iPhone app | Connect the controller to the iPhone using Bluetooth, not to the Mac. | The iPhone app must support the controller. |
| MC.01.05.b104 | Move the pointer to the top of the iPhone Mirroring window and click Home Screen | Go to the iPhone Home Screen. | While iPhone Mirroring is open. |
| MC.01.05.b105 | Click the bar at the bottom of the iPhone Mirroring window | Go to the iPhone Home Screen. | While iPhone Mirroring is open. |
| MC.01.05.b106 | Move the pointer to the top of the iPhone Mirroring window and click App Switcher | Open the iPhone App Switcher. | While iPhone Mirroring is open. |
| MC.01.05.b107 | Click the Search field at the bottom of the Home Screen | Open Spotlight. | In iPhone Mirroring. |
| MC.01.05.b108 | Press Command-4 | Open Control Center. | Requires macOS 27 or later. |
| MC.01.05.b109 | Play iPhone audio | Audio plays through the Mac, and the Mac controls the volume. | Except when listening through headphones connected to the iPhone. |
| MC.01.05.b110 | Play iPhone video | Video plays in the iPhone Mirroring window. | Streaming content providers’ playback restriction policies might affect whether premium video content can be mirrored. |
| MC.01.05.b111 | Attempt to access the iPhone camera or microphone through iPhone Mirroring | Camera and microphone access is unavailable. | Includes Face ID and phone calls. |
| MC.01.05.b112 | Make calls from the Mac while iPhone Mirroring is in use | Make and receive phone calls on the Mac. | Wi-Fi Calling must be turned on on both devices. |
| MC.01.05.b113 | Move the pointer to the top of the iPhone Mirroring window, then drag the appearing toolbar | Move the iPhone Mirroring window. | While the window is open. |
| MC.01.05.b114 | Choose Larger, Smaller, or Actual Size from the View menu | Switch to the selected preset iPhone Mirroring window size. | While iPhone Mirroring is open. |
| MC.01.05.b115 | Drag a corner of the iPhone Mirroring window | Resize the window. | macOS 27 or later. |
| MC.01.05.b116 | Drag to make the window wider | Some apps, including Mail, Messages, and Calendar, show more information. | macOS 27 or later; applies to apps designed to show more information at wider window sizes. |
| MC.01.05.b117 | Click the iPhone size button or press Command-9 | Return to standard iPhone size. | After resizing the window in macOS 27 or later. |
| MC.01.05.b118 | Use an app that requires landscape view | iPhone Mirroring automatically switches to landscape view; it switches back as required by the app. | The orientation follows the app being used. |
| MC.01.05.b119 | Connect the Mac to the iPhone with iPhone Mirroring | The Mac automatically receives notifications and Live Activities from the iPhone. | Reception continues whether or not iPhone Mirroring is being used and whether or not the iPhone is nearby, as long as the iPhone is turned on. |
| MC.01.05.b120 | Drag and drop items between devices | Copy items between devices, such as a video from iPhone Photos to the Mac desktop, a photo from Mac Safari into an iPhone Messages or Mail conversation, or files to and from iPhone Files or Mac iCloud Drive. | Drag and drop is one of the listed transfer methods. |
| MC.01.05.b121 | Use Universal Clipboard or AirDrop | Copy items between devices. | These are listed methods for copying between devices. |
| MC.01.05.b122 | Select Ask every time | iPhone Mirroring asks the user to authenticate before mirroring each time. | Authentication can use the Mac login password, Touch ID on the Mac, or Apple Watch. |
| MC.01.05.b123 | Select Authenticate automatically | The user is no longer asked to authenticate before mirroring. | The Mac login password is required to select this setting; the Lock Screen password-after setting remains Immediately. |
| MC.01.05.b124 | Choose Revoke iPhone Access | Reset setup for this iPhone and remove Allow notifications from iPhone from Mac Notifications settings. | To mirror the iPhone and receive its notifications again, set up iPhone Mirroring again. |
| MC.01.05.b125 | The Mac detects more than one eligible nearby iPhone signed in to the same Apple Account | Choose which iPhone the Mac uses for mirroring, iPhone notifications, and Live Activities. | In macOS Tahoe 26.4 or later, use System Settings > General > AirDrop & Continuity; in earlier macOS versions, use System Settings > Desktop & Dock. The menu appears only when more than one eligible iPhone is detected. |
| MC.01.05.b126 | Turn on iPhone Apps in System Settings > Spotlight | Spotlight or Siri can find apps installed on the iPhone and open them in iPhone Mirroring. | macOS Tahoe 26 or later; iPhone Mirroring must already be set up. |
| MC.01.05.b127 | iPhone Mirroring does not find or connect to the iPhone, or connection is interrupted or times out | Follow instructions in the iPhone Mirroring window and try the listed troubleshooting steps if necessary. | Ensure requirements are met, update and restart both devices, stop intensive Bluetooth or Wi-Fi use on either device, check for wireless interference, check VPN or third-party security software and Mac firewall settings, revoke and set up access again, or sign out of the Apple Account on both devices, restart, and sign back in. |
| MC.01.05.b128 | Use AirPlay instead | View, but do not interact with, the iPhone screen or its streamed video on an AirPlay-compatible device. | Suggested as an alternative to iPhone Mirroring, which is unavailable in the European Union. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.01.05.k001 | Command-1 | Go to the Home Screen. | iPhone Mirroring. |
| MC.01.05.k002 | Command-2 | Open the App Switcher. | iPhone Mirroring. |
| MC.01.05.k003 | Command-3 | Open Spotlight. | iPhone Mirroring. |
| MC.01.05.k004 | Command-4 | Open Control Center. | iPhone Mirroring; requires macOS 27 or later. |
| MC.01.05.k005 | Command–Plus sign (+) | Switch to a preset window size. | iPhone Mirroring; listed under View menu window sizing. |
| MC.01.05.k006 | Command–Minus sign (–) | Switch to a preset window size. | iPhone Mirroring; listed under View menu window sizing. |
| MC.01.05.k007 | Command-0 | Switch to a preset window size. | iPhone Mirroring; listed under View menu window sizing. |
| MC.01.05.k008 | Command-9 | Return to standard iPhone size. | After resizing the window in macOS 27 or later. |
| MC.01.05.k009 | Shift (press and hold) while scrolling | Swipe left or right. | iPhone Mirroring. |

macOS 27 statements on this page:

- MC.01.05.n01: Command-4 opens Control Center in iPhone Mirroring; requires macOS 27 or later.
- MC.01.05.n02: In macOS 27 or later, a corner of the iPhone Mirroring window can be dragged to resize it.
- MC.01.05.n03: In macOS 27 or later, some apps, including Mail, Messages, and Calendar, show more information when the window is made wider.
- MC.01.05.n04: In macOS 27 or later, the iPhone size button or Command-9 returns a resized window to standard iPhone size.

System Settings paths referenced: Mac: Notifications settings (Allow notifications from iPhone); Mac: iPhone Mirroring menu > Settings; Mac: System Settings > Lock Screen > Require password after screen saver begins or display is turned off; Mac, macOS Tahoe 26.4 or later: System Settings > General > AirDrop & Continuity > iPhone pop-up menu; Mac, earlier macOS versions: System Settings > Desktop & Dock > iPhone pop-up menu (below iPhone Widgets); Mac: Apple menu > System Settings > Spotlight > iPhone Apps; iPhone: Settings > General > AirPlay & Continuity > iPhone Mirroring

#### Live Activities

T1 · HIG (developer guidance) · [Apple HIG](https://developer.apple.com/design/human-interface-guidelines/live-activities) · macOS 27 text present: no

On a paired Mac, active Live Activities appear in the menu bar using compact, minimal, and expanded presentations. Clicking one launches iPhone Mirroring to display the app. The page says ended activities can remain in the Mac menu bar for up to four hours, unless a custom dismissal time is set.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.01.05.e137 | Menu bar | other | macOS | Location where active Live Activities automatically appear on a paired Mac; an ended activity may remain here for up to four hours. |
| MC.01.05.e138 | Live Activity | other | Across system locations, including the Mac menu bar | A glanceable display for tracking an activity, event, or task that starts on iPad or iPhone. |
| MC.01.05.e139 | Compact | view | Mac menu bar | One of the presentations used for Live Activities on a paired Mac. |
| MC.01.05.e140 | Minimal | view | Mac menu bar | One of the presentations used for Live Activities on a paired Mac. |
| MC.01.05.e141 | Expanded | view | Mac menu bar | One of the presentations used for Live Activities on a paired Mac. |
| MC.01.05.e142 | iPhone Mirroring | other | Launched when a person clicks a Live Activity on Mac | Displays the app associated with the Live Activity. |

| ID | Context | Guideline | Conditions |
|---|---|---|---|
| MC.01.05.b129 | A Live Activity is active on an iPhone paired with a Mac. | It automatically appears in the Mac menu bar using the compact, minimal, and expanded presentations. | The Mac is paired with the iPhone. |
| MC.01.05.b130 | A person clicks a Live Activity in the Mac menu bar. | iPhone Mirroring launches to display the app. | The Live Activity is on a paired Mac. |
| MC.01.05.b131 | A Live Activity ends. | It is immediately removed from the Dynamic Island and CarPlay; on the Lock Screen, in the Mac menu bar, and in the watchOS Smart Stack, it can remain for up to four hours. | A custom dismissal time may be set; the page recommends choosing one proportional to the activity, often 15 to 30 minutes. |
| MC.01.05.b132 | Designing Live Activity layouts for macOS. | Use the provided iOS dimensions as guidance. | The page lists no separate macOS dimensions. |
| MC.01.05.b133 | Designing a Live Activity for any supported context. | Support compact, minimal, expanded, and Lock Screen presentations, and adapt layouts to different screen sizes and presentations. | The page describes these as general Live Activity requirements; the Mac-specific section identifies compact, minimal, and expanded presentations. |

API names: `isIslandLimitedInWidth`, `ContainerRelativeShape`, `activityActionForegroundColor(_:)`, `padding(_:_:)`, `ActivityFamily.small`

### MC.01.06 Menu bar hiding & customization

Status: **PARTIAL**. Menu Bar pane options captured (Automatically hide and show the menu bar: Always/On Desktop Only/In Full Screen Only/Never).

Also covered by: [Customize the menu bar on Mac](https://support.apple.com/guide/mac-help/mchl4af84660/27/mac/27), [Change Menu Bar settings on Mac](https://support.apple.com/guide/mac-help/mchlad96d366/27/mac/27)


## 14.2 MC.02 Dock


### MC.02.01 Placement/size/magnification

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).


#### Use the Dock on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mh35859/27/mac/27) · macOS 27 text present: no

The Dock provides access to apps and features, and can be customised, rearranged, and used to open items or perform actions.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.02.01.e001 | Dock | view | Mac desktop | A place to access apps and features; by default it is along the bottom edge of the screen. |
| MC.02.01.e002 | Safari | icon | Dock example | Named as an example of an app available from the Dock. |
| MC.02.01.e003 | Bin | icon | Dock example | Named as an example of a feature available from the Dock. |
| MC.02.01.e004 | Finder | icon | Dock example | Its Dock icon can be clicked to open Finder. |
| MC.02.01.e005 | Pages | icon | Dock example | Its Dock icon is given as an example of opening a document by dragging it onto the icon. |
| MC.02.01.e006 | shortcut menu | menu | Displayed for an item in the Dock | Contains actions for the item. |
| MC.02.01.e007 | Show Recents | menu item | Shortcut menu for a Dock item | An example action in the shortcut menu. |
| MC.02.01.e008 | Force Quit | menu item | Shortcut menu for an app icon in the Dock | Quits an app that has stopped responding. |
| MC.02.01.e009 | recently used apps | view | Dock | The Dock can show up to three recently used apps that are not already in it. |
| MC.02.01.e010 | line that separates the recently used apps | other | Dock | Separates recently used apps from the left-side area where apps can be added. |
| MC.02.01.e011 | other line that separates recently used apps | other | Dock | Separates recently used apps from the right-side area where files and folders can be added. |
| MC.02.01.e012 | Downloads | folder | Dock | The Dock includes a Downloads folder by default. |
| MC.02.01.e013 | Remove | indicator | Shown when dragging an item out of the Dock | Indicates that the Dock item can be removed. |
| MC.02.01.e014 | Options | menu item | Shortcut menu for an app icon in the Dock | Contains the Keep in Dock option. |
| MC.02.01.e015 | Keep in Dock | menu item | Options submenu for an app icon in the Dock | Keeps the app icon in the Dock. |
| MC.02.01.e016 | Handoff icon | icon | Near the right end of the Dock | Represents the app being used on an iPhone, iPad, iPod touch, or Apple Watch. |
| MC.02.01.e017 | System Settings | app | Mac | Used to customise the Dock. |
| MC.02.01.e018 | Desktop Dock | setting | System Settings | The settings area to open for Dock customisation. |
| MC.02.01.e019 | Open Desktop Dock settings for me | button | Below the Desktop Dock instruction | A link to open Desktop Dock settings. |
| MC.02.01.e020 | Dock | setting | Desktop Dock settings | The section below which Dock options can be changed. |
| MC.02.01.e021 | double arrow | indicator | Separator line in the Dock | Appears when the pointer is moved over the separator line for resizing. |
| MC.02.01.e022 | red badge | indicator | An icon in the Dock | Indicates that one or more actions need to be taken in an app or System Settings. |
| MC.02.01.e023 | Mail | icon | Dock example | A red badge on the Mail icon indicates new emails to read. |
| MC.02.01.e024 | Left Arrow | other | Keyboard navigation of the Dock | Moves from icon to icon toward the left. |
| MC.02.01.e025 | Right Arrow | other | Keyboard navigation of the Dock | Moves from icon to icon toward the right. |
| MC.02.01.e026 | Return | other | Keyboard navigation of the Dock | Opens the selected item. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.02.01.b001 | Click an app icon in the Dock. | The app opens. | Example: click Finder to open Finder. |
| MC.02.01.b002 | Drag a file over an app icon in the Dock. | The file opens in that app. | Example: drag a Pages document over the Pages icon. |
| MC.02.01.b003 | Command-click an item's icon in the Dock. | The item is shown in the Finder. | The item must be in the Dock. |
| MC.02.01.b004 | Option-click the current app's icon in the Dock. | Switches to the previous app and hides the current app. | The current app's icon must be in the Dock. |
| MC.02.01.b005 | Option-Command-click an app icon in the Dock. | Switches to that app and hides all other apps. | Click the icon of the app to switch to. |
| MC.02.01.b006 | Control-click an item in the Dock. | Displays the item's shortcut menu. | Choose an action, such as Show Recents, or click a filename to open the file. |
| MC.02.01.b007 | Control-click the icon of an app that has stopped responding, then choose Force Quit. | The app is force quit. | Unsaved changes may be lost. |
| MC.02.01.b008 | Drag an app to the left side of, or above, the line that separates the recently used apps. | An alias for the app is placed in the Dock. | Adding an app to the Dock. |
| MC.02.01.b009 | Drag a file or folder to the right side of, or below, the other line that separates recently used apps. | An alias for the item is placed in the Dock. | Adding a file or folder to the Dock. |
| MC.02.01.b010 | Drag a folder to the Dock. | The folder can be viewed as a stack. | The Dock includes a Downloads folder by default. |
| MC.02.01.b011 | Drag an item out of the Dock until Remove is shown. | The item is removed from the Dock. | Only the alias is removed; the actual item remains on the Mac. |
| MC.02.01.b012 | Open an app whose icon was removed from the Dock, then Control-click its icon and choose Options > Keep in Dock. | The app icon is put back in the Dock. | The app remains on the Mac after its Dock icon is removed. |
| MC.02.01.b013 | Drag an item to a new location in the Dock. | The item is rearranged in the Dock. | Applies to Dock items. |
| MC.02.01.b014 | Use Handoff on an iPhone, iPad, iPod touch, or Apple Watch. | The Handoff icon for the app being used appears near the right end of the Dock. | Handoff is in use. |
| MC.02.01.b015 | Go to System Settings and click Desktop Dock. | Opens the Dock customisation settings. | Change the desired options below Dock. |
| MC.02.01.b016 | Change options below Dock in Desktop Dock settings. | Changes how items appear in the Dock, its size, its screen-edge location, or whether it is hidden. | The page gives these as examples of available changes. |
| MC.02.01.b017 | Move the pointer over the separator line in the Dock until a double arrow appears, then click and drag the pointer down or up. | Quickly adjusts the Dock's size. | Use the separator line in the Dock. |
| MC.02.01.b018 | Control-click the separator in the Dock. | Displays a shortcut menu with other actions. | The separator must be in the Dock. |
| MC.02.01.b019 | Press Fn-Control-F3. | Moves keyboard navigation to the Dock. | Use the keyboard to navigate to the Dock. |
| MC.02.01.b020 | Press the Left Arrow or Right Arrow key. | Moves from icon to icon in the Dock. | After moving keyboard focus to the Dock. |
| MC.02.01.b021 | Press Return. | Opens the selected Dock item. | After navigating to an item in the Dock. |
| MC.02.01.b022 | A red badge appears on an icon in the Dock. | Indicates that one or more actions need to be taken in an app or System Settings. | For example, a red badge on Mail indicates new emails to read. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.02.01.k001 | Command-click | Show an item in the Finder. | An item's icon in the Dock. |
| MC.02.01.k002 | Option-click | Switch to the previous app and hide the current app. | The current app's icon in the Dock. |
| MC.02.01.k003 | Option-Command-click | Switch to another app and hide all other apps. | The icon of the app to switch to in the Dock. |
| MC.02.01.k004 | Control-click | Display a shortcut menu. | An item or separator in the Dock; for a nonresponding app, the menu can be used to choose Force Quit. |
| MC.02.01.k005 | Fn-Control-F3 | Move to the Dock. | Keyboard navigation. |
| MC.02.01.k006 | Left Arrow | Move to the previous icon. | After keyboard focus moves to the Dock. |
| MC.02.01.k007 | Right Arrow | Move to the next icon. | After keyboard focus moves to the Dock. |
| MC.02.01.k008 | Return | Open an item. | After selecting an item while navigating the Dock. |
| MC.02.01.k009 | Drag a file over an app icon | Open the file in the app. | An app icon in the Dock. |
| MC.02.01.k010 | Drag an app to the left side of or above the separator | Add an alias for the app to the Dock. | The separator that separates recently used apps. |
| MC.02.01.k011 | Drag a file or folder to the right side of or below the other separator | Add an alias for the file or folder to the Dock. | The other line that separates recently used apps. |
| MC.02.01.k012 | Drag an item out of the Dock | Remove the item from the Dock when Remove is shown. | The actual item remains on the Mac. |
| MC.02.01.k013 | Drag an item to a new location | Rearrange the item. | Within the Dock. |
| MC.02.01.k014 | Move pointer over separator, then click and drag down or up | Adjust the Dock's size. | Wait for the double arrow to appear over the Dock separator. |

System Settings paths referenced: System Settings > Desktop Dock

#### Change Desktop & Dock settings on Mac - Apple Support

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp1119/27/mac/27) · macOS 27 text present: no

The page describes Desktop & Dock options for the Dock, desktop and Stage Manager, widgets, windows, and Mission Control. It explains the effects of the settings and several pointer gestures.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.02.01.e027 | Apple menu | menu | Mac menu bar; referenced in the instructions | Menu named as the starting point for changing these settings. |
| MC.02.01.e028 | Open Desktop & Dock settings for me | other | Near the introductory instructions | Link or control label for opening Desktop & Dock settings. |
| MC.02.01.e029 | Dock | panel | Desktop & Dock settings | Section containing Dock options. |
| MC.02.01.e030 | Size | control | Dock | Slider for changing the Dock size. |
| MC.02.01.e031 | Magnification | control | Dock | Option and slider for magnifying Dock icons when the pointer moves over them. |
| MC.02.01.e032 | Dock position on screen | control | Dock | Option for locating the Dock along the left, bottom, or right screen edge. |
| MC.02.01.e033 | Minimized windows animation | control | Dock | Option for choosing the visual effect used when minimizing a window. |
| MC.02.01.e034 | Window title bar double-click action | control | Dock | Option for choosing what happens when an app window's title bar is double-clicked. |
| MC.02.01.e035 | Fill | menu item | Window title bar double-click action options | Expands a window to fill the space between the menu bar and Dock, or, with a side-positioned Dock, below the menu bar and beside the Dock. |
| MC.02.01.e036 | Zoom | menu item | Window title bar double-click action options | Zooms a window. |
| MC.02.01.e037 | Minimize | menu item | Window title bar double-click action options | Minimizes an app window to the Dock. |
| MC.02.01.e038 | No Action | menu item | Window title bar double-click action options | Does nothing when the title bar is double-clicked. |
| MC.02.01.e039 | Minimize windows into application icon | control | Dock | Option for minimizing an app window to its app icon rather than to an area in the Dock. |
| MC.02.01.e040 | Automatically hide and show the Dock | control | Dock | Option for hiding the Dock when it is not in use. |
| MC.02.01.e041 | Animate opening applications | control | Dock | Option for making app icons bounce when apps open. |
| MC.02.01.e042 | Show indicators for open applications | control | Dock | Option for showing a small dot below an app's Dock icon when the app is open. |
| MC.02.01.e043 | Show suggested and recent apps in Dock | control | Dock | Option for showing suggested and recently opened apps, if not already in the Dock, toward one end of the Dock. |
| MC.02.01.e044 | Desktop & Stage Manager | panel | Desktop & Dock settings | Section containing desktop and Stage Manager options. |
| MC.02.01.e045 | Show Items | control | Desktop & Stage Manager | Option for choosing whether to show desktop items. |
| MC.02.01.e046 | On Desktop | menu item | Show Items options | Shows items on the desktop. |
| MC.02.01.e047 | In Stage Manager | menu item | Show Items options | Shows items on the desktop when Stage Manager is on. |
| MC.02.01.e048 | Show Desktop | control | Desktop & Stage Manager | Option for choosing when clicking the wallpaper moves windows out of the way. |
| MC.02.01.e049 | Always on Wallpaper Click | menu item | Show Desktop options | Clicking the wallpaper moves all windows out of the way to show desktop items and widgets. |
| MC.02.01.e050 | Only in Stage Manager on Click | menu item | Show Desktop options | Wallpaper clicking moves all windows out of the way only when Stage Manager is on. |
| MC.02.01.e051 | Stage Manager | control | Desktop & Stage Manager | Option for organizing apps and windows; can be dimmed if Displays have separate Spaces is off. |
| MC.02.01.e052 | Show recent apps in Stage Manager | control | Desktop & Stage Manager | Option for showing recent apps in Stage Manager. |
| MC.02.01.e053 | Show windows from an application | control | Desktop & Stage Manager | Option for choosing which windows appear when using an app. |
| MC.02.01.e054 | All at Once | menu item | Show windows from an application options | Shows all available windows for an app when switching to it. |
| MC.02.01.e055 | One at a Time | menu item | Show windows from an application options | Shows only the app's most recently used window when switching to it. |
| MC.02.01.e056 | Widgets | panel | Desktop & Dock settings | Section containing widget and default browser options. |
| MC.02.01.e057 | Show Widgets | control | Widgets | Option for choosing whether to show widgets. |
| MC.02.01.e058 | Dim widgets on desktop | control | Widgets | Option for choosing a widget appearance style. |
| MC.02.01.e059 | Automatically | menu item | Dim widgets on desktop options | Automatically switches widgets between monochrome and full-color. |
| MC.02.01.e060 | Always | menu item | Dim widgets on desktop options | Always shows widgets in monochrome. |
| MC.02.01.e061 | Never | menu item | Dim widgets on desktop options | Always shows widgets in full-color. |
| MC.02.01.e062 | iPhone Widgets | control | Widgets | Option for showing iPhone widgets in the widgets browser. |
| MC.02.01.e063 | Default web browser | control | Widgets | Option for selecting an installed Mac web browser as the default. |
| MC.02.01.e064 | Windows | panel | Desktop & Dock settings | Section containing document, window closing, and window tiling options. |
| MC.02.01.e065 | Prefer tabs when opening documents | control | Windows | Option with choices Never, Always, or In Full Screen. |
| MC.02.01.e066 | Never | menu item | Prefer tabs when opening documents options | Documents do not open in tabs under this choice. |
| MC.02.01.e067 | Always | menu item | Prefer tabs when opening documents options | Documents open in tabs under this choice. |
| MC.02.01.e068 | In Full Screen | menu item | Prefer tabs when opening documents options | Documents open in tabs when in full screen under this choice. |
| MC.02.01.e069 | Ask to keep changes when closing documents | control | Windows | Option for being asked whether to save changes when closing documents. |
| MC.02.01.e070 | Close windows when quitting an application | control | Windows | Option for closing app windows when quitting an app. |
| MC.02.01.e071 | Drag windows to left or right edge of screen to tile | control | Windows | Option for tiling windows by dragging them to either screen edge. |
| MC.02.01.e072 | Drag windows to menu bar to fill screen | control | Windows | Option for filling the screen by dragging a window to the menu bar. |
| MC.02.01.e073 | Hold ⌥ key while dragging windows to tile | control | Windows | Option for using the Option key to accelerate window tiling while dragging to a screen edge. |
| MC.02.01.e074 | Tiled windows have margins | control | Windows | Option for leaving space around each tiled window. |
| MC.02.01.e075 | Mission Control | panel | Desktop & Dock settings | Section containing Spaces, Mission Control, shortcut, and Hot Corners options. |
| MC.02.01.e076 | Automatically rearrange Spaces based on most recent use | control | Mission Control | Option for rearranging Spaces to provide quicker access to the desktops used most recently. |
| MC.02.01.e077 | When switching to an application, switch to a Space with open windows for the application | control | Mission Control | Option for switching to a Space containing windows for the app being switched to. |
| MC.02.01.e078 | Group windows by application | control | Mission Control | Option for organizing windows by app in Mission Control. |
| MC.02.01.e079 | Displays have separate Spaces | control | Mission Control | Option for setting up separate Spaces for each display when using Spaces with multiple displays. |
| MC.02.01.e080 | Drag windows to top of screen to enter Mission Control | control | Mission Control | Option for entering Mission Control by dragging a window to the top of the screen. |
| MC.02.01.e081 | Shortcuts | panel | Mission Control | Section for viewing or changing keyboard shortcuts and, depending on the mouse or trackpad, potentially two columns of pop-up menus. |
| MC.02.01.e082 | Hot Corners | panel | Mission Control | Section for choosing actions performed when the pointer moves to screen corners. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.02.01.b023 | Drag the Size slider. | Changes the Dock size. | In Dock settings. |
| MC.02.01.b024 | Move the pointer over Dock icons. | Icons are magnified. | Magnification is enabled; its slider sets the magnification size. |
| MC.02.01.b025 | Choose a Dock position. | Locates the Dock along the left, bottom, or right edge of the screen. | In Dock position on screen. |
| MC.02.01.b026 | Minimize a window. | The chosen visual effect is used. | The effect is selected in Minimized windows animation. |
| MC.02.01.b027 | Double-click an app window's title bar. | The window fills the available space, zooms, minimizes to the Dock, or does nothing. | Depends on the selected Window title bar double-click action; Fill respects the menu bar and Dock space. |
| MC.02.01.b028 | Minimize an app window. | The window is minimized to the app icon in the Dock. | Minimize windows into application icon is on; otherwise it is minimized to an area in the Dock. |
| MC.02.01.b029 | Stop using the Dock. | The Dock is hidden. | Automatically hide and show the Dock is on. |
| MC.02.01.b030 | Move the pointer to the screen edge where a hidden Dock is located. | The Dock appears. | The Dock is hidden by Automatically hide and show the Dock. |
| MC.02.01.b031 | Open an app. | The app icon bounces. | Animate opening applications is on. |
| MC.02.01.b032 | An app is open. | A small dot appears below its Dock icon. | Show indicators for open applications is on. |
| MC.02.01.b033 | View the Dock. | Suggested and recently opened apps appear toward one end of the Dock if they are not already included. | Show suggested and recent apps in Dock is on. |
| MC.02.01.b034 | Choose On Desktop or In Stage Manager for Show Items. | Desktop items are shown in the selected context. | In Stage Manager shows items when Stage Manager is on. |
| MC.02.01.b035 | Turn off the option to show desktop items. | Desktop items are hidden; clicking the desktop shows them for access. | Applies when the relevant Show Items option is off. |
| MC.02.01.b036 | Click the wallpaper. | All windows move out of the way to show desktop items and widgets. | Show Desktop is set to Always on Wallpaper Click. |
| MC.02.01.b037 | Click the wallpaper. | All windows move out of the way to show desktop items and widgets. | Show Desktop is set to Only in Stage Manager on Click and Stage Manager is on. |
| MC.02.01.b038 | Turn on Stage Manager. | Apps and windows are organized to keep the desktop clutter-free and help the user stay focused. | Stage Manager must be available; its option is dimmed if Displays have separate Spaces is off. |
| MC.02.01.b039 | Turn off Show recent apps in Stage Manager. | Recently used apps are hidden; moving the pointer to the left screen edge briefly shows them. | In Stage Manager. |
| MC.02.01.b040 | Switch to an app. | All available windows for that app are shown. | Show windows from an application is set to All at Once. |
| MC.02.01.b041 | Switch to an app. | Only the app's most recently used window is shown. | Show windows from an application is set to One at a Time. |
| MC.02.01.b042 | Click the app on the left again. | The next available window for that app opens. | Show windows from an application is set to One at a Time. |
| MC.02.01.b043 | Choose On Desktop or In Stage Manager for Show Widgets. | Widgets are shown in the selected context. | In Stage Manager shows widgets when Stage Manager is on. |
| MC.02.01.b044 | Turn off the option to show widgets. | Desktop items are hidden; clicking the desktop shows the items for access. | As described for Show Widgets. |
| MC.02.01.b045 | Choose a Dim widgets on desktop style. | Widgets switch automatically between monochrome and full-color, always appear monochrome, or always appear full-color. | Depends on the selected choice: Automatically, Always, or Never. |
| MC.02.01.b046 | Click a link in an email or text message. | The link opens in the default web browser. | Safari is the macOS default unless another installed browser is selected. |
| MC.02.01.b047 | Open a document. | The document opens in a tab instead of a new window. | Depends on Prefer tabs when opening documents: Always, or In Full Screen while in full screen; Never does not prefer tabs. |
| MC.02.01.b048 | Close a document with unsaved changes. | The user is asked whether to save changes. | Ask to keep changes when closing documents is on; by default, changes are automatically saved. |
| MC.02.01.b049 | Quit an app. | The app's windows close and do not reopen automatically the next time the app opens. | Close windows when quitting an application is on. |
| MC.02.01.b050 | Drag a window to the left or right edge of the screen. | The window tiles. | Drag windows to left or right edge of screen to tile is enabled. |
| MC.02.01.b051 | Drag a window to the menu bar. | The window fills the screen. | Drag windows to menu bar to fill screen is enabled. |
| MC.02.01.b052 | Hold Option while dragging a window to the screen edge. | Window tiling is accelerated. | Hold ⌥ key while dragging windows to tile is enabled. |
| MC.02.01.b053 | Tile windows. | Space is left around each tiled window. | Tiled windows have margins is enabled. |
| MC.02.01.b054 | Use Spaces. | The desktops used most recently are given quicker access. | Automatically rearrange Spaces based on most recent use is enabled and additional desktops have been created using Spaces. |
| MC.02.01.b055 | Switch to an application. | The desktop scrolls to a Space with windows open for that app. | When switching to an application, switch to a Space with open windows for the application is enabled and Spaces are in use. |
| MC.02.01.b056 | View multiple app windows in Mission Control. | Windows are organized by app. | Group windows by application is enabled. |
| MC.02.01.b057 | Use Spaces with multiple displays. | Each display has separate Spaces; the Dock is available on all displays. | Displays have separate Spaces is on. |
| MC.02.01.b058 | Use multiple displays with Displays have separate Spaces off. | The Dock appears only on the main display. | Displays have separate Spaces is off. |
| MC.02.01.b059 | Use apps in Split View on other displays or use Stage Manager. | The required separate-Spaces setting must be on. | Displays have separate Spaces must be enabled. |
| MC.02.01.b060 | Drag a window to the top of the screen. | Mission Control opens. | Drag windows to top of screen to enter Mission Control is enabled. |
| MC.02.01.b061 | Press and hold Control, Option, Shift, or Command, or a combination of these keys, while viewing the Shortcuts pop-up menus. | Additional available keyboard shortcuts are displayed. | In the Shortcuts section. |
| MC.02.01.b062 | Move the pointer to a screen corner. | The action assigned to that corner is performed. | A Hot Corners action has been selected for that corner. |
| MC.02.01.b063 | Press and hold Command, Shift, Option, or Control, or a combination of these keys, while viewing Hot Corners pop-up menus. | A modifier key is used with the hot corner. | While setting up a Hot Corners action. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.02.01.k015 | ⌥ (Option) | Accelerates window tiling while dragging a window to the screen edge. | When Hold ⌥ key while dragging windows to tile is enabled. |
| MC.02.01.k016 | Control, Option, Shift, or Command, or a combination of these keys | Displays additional available keyboard shortcuts. | Press and hold while viewing the Shortcuts pop-up menus. |
| MC.02.01.k017 | Command, Shift, Option, or Control, or a combination of these keys | Acts as a modifier key with a Hot Corners action. | Press and hold while viewing Hot Corners pop-up menus. |

System Settings paths referenced: Desktop & Dock settings; Mission Control section

### MC.02.02 App indicators

Status: **PARTIAL**. Background-process indicator only in independent beta reports; release notes reference a fixed bug in it (GAP-B5-02).

Also covered by: [Use the Dock on Mac](https://support.apple.com/guide/mac-help/mh35859/27/mac/27), [Change Desktop & Dock settings on Mac - Apple Support](https://support.apple.com/guide/mac-help/mchlp1119/27/mac/27)


#### Independent reports (MC.02.02)

| ID | Kind | Area | Claim | Source |
|---|---|---|---|---|
| MC.02.02.x01 | new | Dock | In the macOS 27 Golden Gate beta, an app that continues running in the background after its window is closed remains visible in the Dock; its activity indicator changes from black to light gray. (translated) | T4 [iMagazine](https://imagazine.pl/2026/06/10/macos-27-golden-gate-aplikacje-dzialajace-w-tle-dock/) |
| MC.02.02.x02 | new | Dock | Hovering over the icon of an app running in the background displays a label indicating that it is active. (translated) | T4 [iMagazine](https://imagazine.pl/2026/06/10/macos-27-golden-gate-aplikacje-dzialajace-w-tle-dock/) |
| MC.02.02.x03 | new | Dock | The contextual menu for the icon of an app running in the background includes an option to stop that activity. (translated) | T4 [iMagazine](https://imagazine.pl/2026/06/10/macos-27-golden-gate-aplikacje-dzialajace-w-tle-dock/) |
| MC.02.02.x04 | new | Dock | The Dock shows background processes in its context menus. | T3 [Wikipedia](https://en.wikipedia.org/wiki/MacOS_Golden_Gate) |
| MC.02.02.x05 | changed | Dock | In the current developer beta, a light gray dot beneath a Dock icon indicates that an app is still running in the background after being quit. Previously, the activity indicator disappeared for quit apps, or the icon disappeared from the Dock entirely. (translated) | T4 [iFun](https://www.ifun.de/macos-27-dock-zeigt-aktive-hintergrund-apps-an-285370/) |
| MC.02.02.x06 | new | Dock | According to the article, a tooltip that appears when you move the pointer over the Dock icon explicitly indicates that the app is running in the background. (translated) | T4 [iFun](https://www.ifun.de/macos-27-dock-zeigt-aktive-hintergrund-apps-an-285370/) |
| MC.02.02.x07 | new | Dock | The app’s background activity can be stopped from the Dock icon’s context menu. This stops the running process and removes the indicator from the Dock. The name of the new menu item is not given. (translated) | T4 [iFun](https://www.ifun.de/macos-27-dock-zeigt-aktive-hintergrund-apps-an-285370/) |
| MC.02.02.x08 | bug | Dock | A commenter reports that the gray dot disappeared in their macOS 27 beta 4 and that every app has a gray dot for them; they suspect a bug. This is a user report, not a confirmed statement from the article. (translated) | T4 [iFun](https://www.ifun.de/macos-27-dock-zeigt-aktive-hintergrund-apps-an-285370/) |
| MC.02.02.x09 | new | Dock | According to the article, in macOS 27, if an app’s background process remains after the app is quit, the Dock indicator does not disappear but turns into a light gray dot. (translated) | T4 [apple-hacks](https://www.apple-hacks.com/entry/macos-27-golden-gate-background-apps-dock-indicator) |
| MC.02.02.x10 | behavior | Dock | According to the article, running apps have a black dot beneath their Dock icon, and the dot disappears when they are quit. (translated) | T4 [apple-hacks](https://www.apple-hacks.com/entry/macos-27-golden-gate-background-apps-dock-indicator) |
| MC.02.02.x11 | new | Dock | According to the article, hovering over a Dock icon with a gray dot displays text indicating that it is running in the background. (translated) | T4 [apple-hacks](https://www.apple-hacks.com/entry/macos-27-golden-gate-background-apps-dock-indicator) |
| MC.02.02.x12 | new | Dock | According to the article, you can secondary-click an icon with a gray dot and choose a menu item to terminate the background process and remove the icon from the Dock. (translated) | T4 [apple-hacks](https://www.apple-hacks.com/entry/macos-27-golden-gate-background-apps-dock-indicator) |
| MC.02.02.x13 | new | other | According to the article, open “一般” (General) in System Settings, then “ログイン項目と拡張機能” (Login Items & Extensions) and “バックグラウンドアプリのアクティビティ” (Background App Activity) to toggle permission for background activity on a per-app basis. (translated) | T4 [apple-hacks](https://www.apple-hacks.com/entry/macos-27-golden-gate-background-apps-dock-indicator) |
| MC.02.02.x14 | new | other | According to the article, “バックグラウンドアプリのアクティビティ” (Background App Activity) lists apps running in the background, including apps that are not shown in the Dock. (translated) | T4 [apple-hacks](https://www.apple-hacks.com/entry/macos-27-golden-gate-background-apps-dock-indicator) |
| MC.02.02.x15 | new | Dock | The Dock can continue to show apps that have been quit but are still running in the background. | T4 [Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/) |
| MC.02.02.x16 | new | Dock | For apps shown as running in the background, the contextual menu displays that status and offers an option to stop the background processing. | T4 [Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/) |
| MC.02.02.x17 | bug | Dock | The reviewer reports that some apps, including Shortcuts, remain in the Dock after being fully quit, and says it is unclear why some apps appear in this state and others do not. | T4 [Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/) |
| MC.02.02.x18 | behavior | Dock | Thaw leaves system binaries with no LaunchAgent claim alone because it has no way to bring them back after stopping them. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.02.02.x19 | changed | Dock | Apple's app icons have additional Liquid Glass layers and sharper edges across light, dark, clear, and tinted appearances, including in the Dock and app grid. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MC.02.02.x20 | new | Dock | When an app continues a background process after being quit, its Dock indicator turns light gray. Users can end the activity from the app icon's context menu. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MC.02.02.x21 | behavior | Dock | Full Keyboard Access can move focus to the Dock with Fn-A. | T4 [Lets Suite](https://www.lets-suite.com/blog/full-keyboard-access-mac) |
| MC.02.02.x22 | new | Dock | The Siri AI app can be opened by clicking it in the Dock. | T4 [Tom's Guide](https://www.tomsguide.com/computing/macos/macos-golden-gate-review) |
| MC.02.02.x23 | bug | Dock | Fixed an issue where the Dock could remain visible when entering or exiting Mission Control while using an app in full screen. | T4 [BetaProfile](https://betaprofile.dev/macos-27-rc/) |
| MC.02.02.x24 | bug | Dock | Fixed an issue where the Dock could incorrectly indicate that some applications were running in the background. | T4 [BetaProfile](https://betaprofile.dev/macos-27-rc/) |

### MC.02.03 Context menu incl. background processes (27)

Status: **PARTIAL**. Dock shortcut menu items documented; the new background-activity menu item name is unverified (GAP-B5-02).

Also covered by: [Use the Dock on Mac](https://support.apple.com/guide/mac-help/mh35859/27/mac/27), [macOS 27 Golden Gate Release Notes](https://developer.apple.com/documentation/macos-release-notes/macos-27-release-notes)


#### Dock menus

T1 · HIG (developer guidance) · [Apple HIG](https://developer.apple.com/design/human-interface-guidelines/dock-menus) · macOS 27 text present: no

Dock menus appear when someone secondary-clicks an app or game icon in the Dock. They can contain system-provided and custom items, with system-provided items varying depending on whether the app is open.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.02.03.e001 | Dock menu | menu | Shown from an app’s or game’s icon in the Dock | Contains system-provided and custom items. |
| MC.02.03.e002 | Dock | other | At the bottom or side of the Mac interface | Contains app and game icons that can be secondary-clicked to reveal Dock menus. |
| MC.02.03.e003 | Safari | other | Dock menu example | Its Dock menu can include items for viewing a current window or creating a new window. |
| MC.02.03.e004 | Mail | other | Dock menu example | Its Dock menu can include items for getting new mail, composing a new message, and listing open windows. |
| MC.02.03.e005 | Home Screen quick actions | menu | iOS and iPadOS Home Screen or Dock | A similar menu of system-provided and custom items, revealed by long-pressing an app icon. Not supported on macOS. |
| MC.02.03.e006 | Menus | other | Related guidance | Related guidance for labeling and organizing menu items. |
| MC.02.03.e007 | application | other | AppKit developer documentation | A related developer documentation symbol. |

| ID | Context | Guideline | Conditions |
|---|---|---|---|
| MC.02.03.b001 | Someone secondary-clicks an app’s or game’s icon in the Dock | A Dock menu appears with system-provided and custom items. | On a Mac. |
| MC.02.03.b002 | The app’s open state changes | The system-provided items in its Dock menu can vary. | For example, Safari’s menu can include actions for viewing a current window or creating a new window. |
| MC.02.03.b003 | Designing custom Dock menu items | Label items succinctly, organize them logically, make the same commands available elsewhere, and prefer high-value items. | The guidance suggests listing open or recently open windows and a few useful actions, including when the app is not frontmost or has no open windows. |

API names: `application`

### MC.02.04 Stacks

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).


#### Use desktop stacks on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mh35846/27/mac/27) · macOS 27 text present: no

Desktop stacks organize files on the Mac desktop into groups. Files saved to the desktop are automatically added to an appropriate stack.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.02.04.e001 | desktop | view | Mac desktop | The area where desktop stacks organize files. |
| MC.02.04.e002 | View | menu | Menu bar, after clicking the desktop | Contains commands for using and configuring desktop stacks. |
| MC.02.04.e003 | Use Stacks | menu item | View menu; also available in the desktop Control-click menu | Turns desktop stacks on. |
| MC.02.04.e004 | Group Stacks By | menu item | View menu and desktop Control-click menu | Opens choices for grouping stacks. |
| MC.02.04.e005 | kind | menu item | Group Stacks By choices | Groups stacks by kind, such as images or PDFs. |
| MC.02.04.e006 | date | menu item | Group Stacks By choices | Groups stacks by date, such as when a file was last opened or created. |
| MC.02.04.e007 | Finder tags | menu item | Group Stacks By choices | Groups stacks by Finder tags. |
| MC.02.04.e008 | Show View Options | menu item | View menu and desktop Control-click menu | Opens options for changing the appearance of desktop stacks. |
| MC.02.04.e009 | Down Arrow | icon | Expanded desktop stack | Clicking the icon collapses the stack. |
| MC.02.04.e010 | Dock | other | Mac | Folders in the Dock can be displayed as stacks. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.02.04.b001 | Save a file to the desktop | The file is automatically added to the appropriate stack. | Desktop stacks are in use. |
| MC.02.04.b002 | Click the desktop, then choose View > Use Stacks | Desktop stacks are turned on. | On a Mac. |
| MC.02.04.b003 | Control-click the desktop, then choose Use Stacks | Desktop stacks are turned on. | On a Mac. |
| MC.02.04.b004 | Swipe left or right on a stack using two fingers | Browse files in the stack. | Using a trackpad. |
| MC.02.04.b005 | Swipe left or right on a stack using one finger | Browse files in the stack. | Using a Magic Mouse. |
| MC.02.04.b006 | Click a stack on the desktop | The stack expands. | On a Mac. |
| MC.02.04.b007 | Double-click an item in an expanded stack | The item opens. | The stack is expanded. |
| MC.02.04.b008 | Click the stack’s Down Arrow icon | The stack collapses. | The stack is expanded. |
| MC.02.04.b009 | Click the desktop, choose View > Group Stacks By, then choose an option | Stacks are grouped by the selected option. | Available grouping types include kind, date, and Finder tags. |
| MC.02.04.b010 | Control-click the desktop, choose Group Stacks By, then choose an option | Stacks are grouped by the selected option. | Available grouping types include kind, date, and Finder tags. |
| MC.02.04.b011 | Click the desktop, choose View > Show View Options, then change options | The appearance of desktop stacks can be changed, including icon size, spacing between icons, label placement, and whether more information is shown. | On a Mac. |
| MC.02.04.b012 | Control-click the desktop, choose Show View Options, then change options | The appearance of desktop stacks can be changed, including icon size, spacing between icons, label placement, and whether more information is shown. | On a Mac. |
| MC.02.04.b013 | Display a folder in the Dock as a stack | The folder is displayed as a stack in the Dock. | The page refers readers to “Use folders in the Dock.” |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.02.04.k001 | Control-Command-0 | Turn on desktop stacks. | On the Mac desktop. |

### MC.02.05 Minimize effects

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).

Also covered by: [Change Desktop & Dock settings on Mac - Apple Support](https://support.apple.com/guide/mac-help/mchlp1119/27/mac/27)


#### Move and arrange app windows on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp2469/27/mac/27) · macOS 27 text present: no

The page describes how to move, align, merge, maximize, minimize, resize, switch between, and close app windows on Mac. It also explains that closing windows does not quit the app.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.02.05.e001 | Finder | other | App window introduction | Named as an app whose window can open on the desktop. |
| MC.02.05.e002 | desktop | view | App window introduction | The area where an app or Finder window opens. |
| MC.02.05.e003 | menu bar | view | Top of the screen | Shows the active app’s name in bold and its app menus. |
| MC.02.05.e004 | title bar | view | Window | Can be dragged to move a window; double-click behavior can be configured in Desktop & Dock settings. |
| MC.02.05.e005 | green button | button | Window | Holding the pointer over it is mentioned for moving or maximizing a window. |
| MC.02.05.e006 | yellow minimize button | button | Window | Click it to minimize a window. |
| MC.02.05.e007 | Window | menu | App menu bar | Contains the Merge All Windows and Move Tab to New Window commands. |
| MC.02.05.e008 | Merge All Windows | menu item | Window menu | Merges an app’s windows into one tabbed window. |
| MC.02.05.e009 | Move Tab to New Window | menu item | Window menu | Moves the selected tab into a separate window. |
| MC.02.05.e010 | Fill | setting | Desktop & Dock settings | The setting value specified for double-clicking a window’s title bar to maximize it. |
| MC.02.05.e011 | Desktop & Dock | setting | System Settings | Settings area referenced for title-bar double-click behavior. |
| MC.02.05.e012 | Dock | view | Screen edge | A small dot below an app’s icon indicates that the app remains open. |
| MC.02.05.e013 | Mission Control | other | Mac | Can arrange open windows and spaces in a single layer. |
| MC.02.05.e014 | Stage Manager | other | See also | Listed as a related topic. |
| MC.02.05.e015 | close button | button | Window | The page shows a close control for closing a single window; its label is not provided in the text. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.02.05.b001 | Open an app or Finder | A window opens on the desktop. | On Mac. |
| MC.02.05.b002 | An app is active | Its name appears in bold and its menus appear in the menu bar. | Only one app at a time is active. |
| MC.02.05.b003 | Drag a window by its title bar | Moves the window to the desired location. | Some windows cannot be moved. |
| MC.02.05.b004 | Hold the pointer over the green button | The page describes this action under moving a window to one side of the screen. | The specific resulting control or placement is not stated. |
| MC.02.05.b005 | Hold the pointer over the green button | The page describes this action under moving a window to the top or bottom of the screen. | The specific resulting control or placement is not stated. |
| MC.02.05.b006 | Drag a window close to another window | The window aligns without overlapping; multiple windows can be positioned adjacent to each other. | As the window nears the other window. |
| MC.02.05.b007 | Drag an edge of a window toward the edge of an adjacent window | The edges align and resizing stops, making adjacent windows the same size. | As the dragged edge nears the adjacent window’s edge. |
| MC.02.05.b008 | Choose Window > Merge All Windows | The app’s windows are merged into one tabbed window. | If the app has more than one type of window, only the active type is merged. |
| MC.02.05.b009 | Select a tab, then choose Window > Move Tab to New Window | The tab becomes a separate window. | In an app with a tabbed window. |
| MC.02.05.b010 | Drag a tab out of its window | The tab becomes a separate window. | In an app with a tabbed window. |
| MC.02.05.b011 | Hold the pointer over the green button | Maximizes the window. | In a window; the page does not specify further details of the interaction. |
| MC.02.05.b012 | Double-click a window’s title bar | Maximizes the window. | Only if “Double-click a window’s title bar to” is set to “Fill” in Desktop & Dock settings. |
| MC.02.05.b013 | Click the yellow minimize button | Minimizes the window. | In a window. |
| MC.02.05.b014 | Double-click a window’s title bar | Minimizes the window. | If the option to minimize a window when its title bar is double-clicked is set in Desktop & Dock settings. |
| MC.02.05.b015 | Drag a window’s top, bottom, or side edge | Resizes the window. | Most windows can be manually resized. |
| MC.02.05.b016 | Double-click a window’s edge | Expands that side of the window. | Most windows can be manually resized. |
| MC.02.05.b017 | Press Command-Tab | Switches to the previous app. | On Mac. |
| MC.02.05.b018 | Hold Command, press Tab, then press Left or Right arrow until the desired app is reached; release Command | Scrolls through open apps and switches to the selected app. | On Mac. |
| MC.02.05.b019 | Press Esc or Period while scrolling through apps | Cancels the app switch; then release Command. | If the user changes their mind while scrolling. |
| MC.02.05.b020 | Click the close control in a window | Closes that single window. | In a window. |
| MC.02.05.b021 | Close one or all windows for an app | The app does not quit and remains open, indicated by a small dot below its Dock icon. | Closing windows is distinct from quitting the app. |
| MC.02.05.b022 | Use Mission Control | Arranges open windows and spaces in a single layer so the desired one can be spotted. | On Mac. |
| MC.02.05.b023 | Press Command-H | Hides the active app. | On Mac. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.02.05.k001 | Command-Tab | Switch to the previous app. | On Mac. |
| MC.02.05.k002 | Hold Command, press Tab, then press Left Arrow or Right Arrow; release Command | Scroll through open apps and switch to the selected app. | App switching. |
| MC.02.05.k003 | Esc | Cancel scrolling through apps. | While holding Command and scrolling through apps; release Command afterward. |
| MC.02.05.k004 | Period | Cancel scrolling through apps. | While holding Command and scrolling through apps; release Command afterward. |
| MC.02.05.k005 | Option-Command-W | Close all open windows for an app. | Does not quit the app. |
| MC.02.05.k006 | Command-Q | Quit the app. | On Mac. |
| MC.02.05.k007 | Command-H | Hide the active app. | On Mac. |

System Settings paths referenced: System Settings > Desktop & Dock

## 14.3 MC.03 Finder


### MC.03.01 Windows/tabs/views

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).


#### Change how folders open in the Finder on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp1032/27/mac/27) · macOS 27 text present: no

The page explains how to set whether folders open in Finder tabs or windows, how to open an individual folder, and how to work with tabs.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.03.01.e001 | System Settings | app | Step 1 | App to open to change how folders open. |
| MC.03.01.e002 | Desktop & Dock | setting | System Settings | Settings area to click. |
| MC.03.01.e003 | Open Desktop & Dock settings for me | other | Below the instruction to click Desktop & Dock | Link to open Desktop & Dock settings. |
| MC.03.01.e004 | Windows | panel | Right side of Desktop & Dock settings | Section containing the folder-opening preference. |
| MC.03.01.e005 | Prefer tabs when opening documents | control | Windows section | Pop-up menu for choosing how folders open. |
| MC.03.01.e006 | Never | menu item | Prefer tabs when opening documents menu | Opens folders in a new window, not a tab. |
| MC.03.01.e007 | Always | menu item | Prefer tabs when opening documents menu | Opens folders in a new tab. |
| MC.03.01.e008 | In Full Screen | menu item | Prefer tabs when opening documents menu | Opens folders in a tab only when working in full screen. |
| MC.03.01.e009 | Finder | app | Throughout the page | App in which folders and tabs are opened. |
| MC.03.01.e010 | Finder > Settings | menu | Finder | Menu path for Finder settings. |
| MC.03.01.e011 | General | panel | Finder Settings | Settings section containing the tab preference. |
| MC.03.01.e012 | Open folders in tabs instead of new windows | setting | Finder Settings > General | Select to have folders open in tabs; deselect to have them open in windows. |
| MC.03.01.e013 | Finder toolbar | view | Finder | The tip describes behavior when this and the sidebar are hidden. |
| MC.03.01.e014 | sidebar | view | Finder | The tip describes behavior when this and the Finder toolbar are hidden. |
| MC.03.01.e015 | Dock | menu | Finder instructions | Named before the File menu in the instruction for opening a new Finder window. |
| MC.03.01.e016 | File | menu | Finder | Menu containing New Finder Window. |
| MC.03.01.e017 | New Finder Window | menu item | File menu | Opens a new Finder window without opening a specific folder. |
| MC.03.01.e018 | tabs | view | Finder | The page describes scrolling through tabs and closing a tab. |
| MC.03.01.e019 | See also | other | End of page | Related links section. |
| MC.03.01.e020 | Sort and arrange items in the Finder on Mac | other | See also | Related support link. |
| MC.03.01.e021 | Organize files in folders on Mac | other | See also | Related support link. |
| MC.03.01.e022 | Change Finder settings on Mac | other | See also | Related support link. |
| MC.03.01.e023 | Customize the Finder toolbar on Mac | other | See also | Related support link. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.03.01.b001 | Open a folder in Finder | Usually, the folder’s contents replace the current contents of the window. | Default behavior described by the page. |
| MC.03.01.b002 | Choose Never from Prefer tabs when opening documents | Folders open in a new window, not a tab. | Set in System Settings > Desktop & Dock > Windows. |
| MC.03.01.b003 | Choose Always from Prefer tabs when opening documents | Folders open in a new tab. | Set in System Settings > Desktop & Dock > Windows. |
| MC.03.01.b004 | Choose In Full Screen from Prefer tabs when opening documents | Folders open in a tab. | Only when working in full screen; set in System Settings > Desktop & Dock > Windows. |
| MC.03.01.b005 | Select Open folders in tabs instead of new windows | Folders open in tabs. | Finder > Settings > General. |
| MC.03.01.b006 | Deselect Open folders in tabs instead of new windows | Folders open in windows. | Finder > Settings > General. |
| MC.03.01.b007 | Open a single folder in Finder | The folder opens in a new tab or window. | Depends on Finder settings. |
| MC.03.01.b008 | Double-click a folder without pressing the Command key | The folder opens in a new window. | Finder toolbar and sidebar are hidden. |
| MC.03.01.b009 | Choose File > New Finder Window | A new Finder window opens without opening a specific folder. | In Finder. |
| MC.03.01.b010 | Scroll through the tabs | You can see tabs that are not all visible. | In Finder, when all tabs are not visible. |
| MC.03.01.b011 | Place the pointer over a tab, then click the close control | The tab closes. | In Finder. The source shows no label for the close control. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.03.01.k001 | Command-N | Open a new Finder window without opening a specific folder. | Finder. |

System Settings paths referenced: System Settings > Desktop & Dock > Windows > Prefer tabs when opening documents; Finder > Settings > General > Open folders in tabs instead of new windows

#### Change Finder settings on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp2803/27/mac/27) · macOS 27 text present: yes

Finder settings let you customize Finder windows, tags, the sidebar, filename extensions, Trash behavior, folder sorting, and search scope.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.03.01.e024 | Dock | other | Mac interface | The article says to click Dock to access Finder settings. |
| MC.03.01.e025 | Finder | menu | Menu bar | Choose Finder to access its settings. |
| MC.03.01.e026 | Settings | menu item | Finder menu | Opens Finder settings. |
| MC.03.01.e027 | General | panel | Finder settings | Contains general Finder options. |
| MC.03.01.e028 | Show these items on the desktop | setting | General | Controls whether connected hardware appears on the desktop. |
| MC.03.01.e029 | New Finder windows show | setting | General | Chooses the folder displayed when a new Finder window opens. |
| MC.03.01.e030 | Sync Desktop & Documents folders | setting | General | Provides access to Documents and Desktop folders on other devices using iCloud. |
| MC.03.01.e031 | Open folders in tabs instead of new windows | setting | General | Controls whether folders open in tabs instead of windows when opened with the described Command-key action. |
| MC.03.01.e032 | Suggest file names | setting | General | With Apple Intelligence, provides filename suggestions when renaming, saving, or editing files. |
| MC.03.01.e033 | Tags | panel | Finder settings | Contains tag display, favorite tag, and folder tint options. |
| MC.03.01.e034 | Show these tags in the sidebar | setting | Tags | Customizes which tags appear in the Finder sidebar. |
| MC.03.01.e035 | Favorite Tags | view | Tags | Lists tags used in the shortcut menu shown when Control-clicking a file. |
| MC.03.01.e036 | Tint folders based on tags | setting | Tags | Adds a color tint to a folder to indicate that it has a tag. |
| MC.03.01.e037 | Sidebar | panel | Finder settings | Contains options for items shown in the Finder sidebar. |
| MC.03.01.e038 | Show these items in the sidebar | setting | Sidebar | Controls which frequently used or quickly accessed items appear in the Finder sidebar. |
| MC.03.01.e039 | Advanced | panel | Finder settings | Contains filename extension, warning, Trash, folder sorting, and search options. |
| MC.03.01.e040 | Show all filename extensions | setting | Advanced | Controls whether all filename extensions are shown, including extensions for files with “Hide extension” selected. |
| MC.03.01.e041 | Hide extension | setting | Individual file settings | An individual file option that affects whether its filename extension is shown when “Show all filename extensions” is deselected. |
| MC.03.01.e042 | Show warning before changing an extension | setting | Advanced | Controls whether a warning appears when an extension is about to be changed. |
| MC.03.01.e043 | Show warning before removing from iCloud Drive | setting | Advanced | Controls whether a warning appears before items are removed from iCloud Drive. |
| MC.03.01.e044 | Show warning before emptying the Trash | setting | Advanced | Controls whether a warning appears before items in the Trash are deleted. |
| MC.03.01.e045 | Remove items from the Trash after 30 days | setting | Advanced | Controls whether items are permanently removed 30 days after being moved to the Trash. |
| MC.03.01.e046 | Keep folders on top: In windows when sorting by name | setting | Advanced | Controls whether folders are listed alphabetically before files when a Finder window is sorted by name. |
| MC.03.01.e047 | Keep folders on top: On Desktop | setting | Advanced | Controls whether folders are listed alphabetically before files on the desktop. |
| MC.03.01.e048 | When performing a search | setting | Advanced | Chooses the default search scope for the Finder window search field. |
| MC.03.01.e049 | Open and Save dialogs | view | Open and Save dialogs | The items selected for the Finder sidebar also appear in these dialogs. |
| MC.03.01.e050 | shortcut menu | menu | Shown when Control-clicking a file | The menu can show up to seven favorite tags. |
| MC.03.01.e051 | Finder window | window | Finder | A window in which the user can search, sort, and view files and folders. |
| MC.03.01.e052 | Trash | other | Finder | Items moved here can be permanently removed immediately or after 30 days, depending on the setting. |
| MC.03.01.e053 | iCloud Drive | other | Finder | The article describes a warning setting for removing items from iCloud Drive. |
| MC.03.01.e054 | Apple Intelligence | other | Filename suggestions and article notes | Used for file-name suggestions; availability and usage-limit notes are included. |
| MC.03.01.e055 | Command key | control | Keyboard | Used while double-clicking a folder for the tab-opening behavior described in General. |
| MC.03.01.e056 | Control-click | gesture | Finder file | Opens the shortcut menu used to access favorite tags. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.03.01.b012 | Click Dock, then choose Finder > Settings. | Opens Finder settings. | On a Mac. |
| MC.03.01.b013 | Enable “Show these items on the desktop.” | Connected hardware is shown on the desktop. | The setting is in General. |
| MC.03.01.b014 | Choose a folder for “New Finder windows show.” | That folder is displayed when a new Finder window opens. | The setting is in General. |
| MC.03.01.b015 | Use “Sync Desktop & Documents folders.” | Desktop and Documents folders can be accessed on other devices using iCloud. | The setting is in General. |
| MC.03.01.b016 | Press and hold the Command key and double-click a folder. | The folder opens in a tab instead of a new window. | “Open folders in tabs instead of new windows” is the relevant General option. |
| MC.03.01.b017 | Rename, save, or edit a file. | Apple Intelligence can show filename suggestions. | Use “Suggest file names”; Apple Intelligence availability may vary by language or region, and usage limits may apply. |
| MC.03.01.b018 | Select the checkbox to the right of a tag. | The tag appears in the Finder sidebar. | In Tags, under “Show these tags in the sidebar.” |
| MC.03.01.b019 | Click the color next to a tag, then choose a new color. | Changes the tag color. | In Tags. |
| MC.03.01.b020 | Click a tag, click its name, then enter a new name. | Renames the tag. | In Tags. |
| MC.03.01.b021 | Click the add control. | Creates a new tag. | In Tags; the article does not provide a text label for the control. |
| MC.03.01.b022 | Select a tag, then click the delete control. | Deletes the tag. | In Tags; the article does not provide a text label for the control. |
| MC.03.01.b023 | Select a tag in the list above Favorite Tags, then drag it over a tag to replace it. | Adds the selected tag to Favorite Tags in place of the tag it is dragged over. | A maximum of seven tags can appear in the shortcut menu. |
| MC.03.01.b024 | Drag a tag out of Favorite Tags until the removal indicator appears. | Removes the tag from Favorite Tags. | The removal indicator is depicted but has no text label in the article. |
| MC.03.01.b025 | Enable “Tint folders based on tags.” | Adds a color tint to a folder to indicate that it has a tag. | In Tags. |
| MC.03.01.b026 | Choose items under “Show these items in the sidebar.” | Shows those items in the Finder sidebar and in Open and Save dialogs. | In Sidebar. |
| MC.03.01.b027 | Enable “Show all filename extensions.” | Shows all file extensions, including for files with “Hide extension” selected. | In Advanced; the option affects most, but not all, filenames. |
| MC.03.01.b028 | Deselect “Show all filename extensions.” | Shows or hides extensions according to each file’s individual “Hide extension” setting. | In Advanced. |
| MC.03.01.b029 | Attempt to change a filename extension. | Displays a warning if “Show warning before changing an extension” is enabled. | The article advises that filename extensions generally should not be changed. |
| MC.03.01.b030 | Remove an item from iCloud Drive. | Displays a warning if “Show warning before removing from iCloud Drive” is enabled. | In Advanced. |
| MC.03.01.b031 | Empty the Trash. | Displays a warning if “Show warning before emptying the Trash” is enabled. | In Advanced. |
| MC.03.01.b032 | An item has been in the Trash for 30 days. | The item is removed permanently if “Remove items from the Trash after 30 days” is enabled. | In Advanced. |
| MC.03.01.b033 | Sort a Finder window by name. | Folders are shown alphabetically first, followed by files alphabetically. | “Keep folders on top: In windows when sorting by name” is enabled. |
| MC.03.01.b034 | View items on the desktop. | Folders are shown alphabetically first, followed by files alphabetically. | “Keep folders on top: On Desktop” is enabled. |
| MC.03.01.b035 | Use the search field in a Finder window. | Searches the entire Mac, the currently open folder, or the scope specified the last time a search was performed. | The default scope is chosen with “When performing a search.” |
| MC.03.01.b036 | Control-click a file. | Shows a shortcut menu that can include favorite tags. | Favorite Tags can contain up to seven tags. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.03.01.k002 | Command key + double-click | Opens a folder in a tab instead of a new window. | When “Open folders in tabs instead of new windows” is enabled. |
| MC.03.01.k003 | Control-click | Opens the shortcut menu for a file. | The menu can display Favorite Tags. |

macOS 27 statements on this page:

- MC.03.01.n01: The “Suggest file names” option uses Apple Intelligence to provide filename suggestions when renaming, saving, or editing files.
- MC.03.01.n02: The article states that Apple Intelligence on macOS 27 is not available in all languages or regions and that usage limits may apply.
- MC.03.01.n03: The article states that Siri AI is available in beta in macOS 27, along with iOS 27, iPadOS 27, watchOS 27, and visionOS 27; this is in the general Apple Intelligence note, not a Finder-specific setting.

#### Use the Finder on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp2605/27/mac/27) · macOS 27 text present: no

The page describes using Finder to view and organize files, customize views and icons, create aliases and Smart Folders, and sync content with connected Apple devices.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.03.01.e057 | Dock | other | Mac desktop | Clicking the Dock opens a Finder window. |
| MC.03.01.e058 | Finder window | window | Finder | A window for viewing and working with items on the Mac. |
| MC.03.01.e059 | Finder sidebar | view | Finder window | Shows items such as apps, documents, and downloads; it can be customized. |
| MC.03.01.e060 | Preview pane | panel | Finder window | Lets you quickly view a document or image without opening it. |
| MC.03.01.e061 | Gallery view | view | Finder window | Displays items visually so you can flip through files and folders. |
| MC.03.01.e062 | Date Last Opened | menu item | Finder item-arrangement options | An example of an option for grouping items. |
| MC.03.01.e063 | Smart Folder | other | Finder | Automatically collects documents based on criteria you set. |
| MC.03.01.e064 | File | menu | Finder menu bar | The menu used with Make Alias. |
| MC.03.01.e065 | Make Alias | menu item | File menu | Creates an alias for a selected file, folder, or app. |
| MC.03.01.e066 | Double-click | gesture | Finder | Used to open an alias. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.03.01.b037 | Click Dock | A Finder window opens. | On a Mac. |
| MC.03.01.b038 | Click an item in the Finder sidebar | Finder shows the corresponding items, such as apps, documents, or downloads. | The item is shown in the sidebar. |
| MC.03.01.b039 | Customize the Finder sidebar | Change or rearrange the items it shows, or add an item you open frequently. | — |
| MC.03.01.b040 | Use the Preview pane | Quickly view a document or image without opening it. | In a Finder window. |
| MC.03.01.b041 | Choose Gallery view | View files and folders visually and flip through them. | In a Finder window. |
| MC.03.01.b042 | Sort items by name | Keep folders at the top of the list in alphabetical order. | When sorting by name. |
| MC.03.01.b043 | Click the item-arrangement control and choose an option such as Date Last Opened | Group items using the chosen option. | In Finder; the control's label is not specified in the provided text. |
| MC.03.01.b044 | Create and organize folders | Keep related files together. | In Finder. |
| MC.03.01.b045 | Tag files or folders with keywords | Locate them by searching for a keyword or clicking the tag in the Finder sidebar. | The files or folders have been tagged. |
| MC.03.01.b046 | Set criteria for a Smart Folder | The Smart Folder automatically collects matching documents. | Examples given are spreadsheets or documents with “Paris Vacation” in the title. |
| MC.03.01.b047 | Customize an item's icon | Change a folder icon's appearance with colors, symbols, or emoji, or create a custom icon for a file or folder. | In Finder. |
| MC.03.01.b048 | Select a file, folder, or app, then choose File > Make Alias | Create an alias that can be placed in a convenient location. | The selected item is a file, folder, or app. |
| MC.03.01.b049 | Double-click an alias | Open the item the alias refers to. | — |
| MC.03.01.b050 | Connect an iPhone, iPad, or iPod touch to the Mac | The device appears in the Finder sidebar, where you can choose items to sync. | The device is connected to the Mac. |
| MC.03.01.b051 | Add a movie to the Mac and sync with an iPhone | The movie can be watched on both the Mac and iPhone. | The movie is synced with the iPhone. |
| MC.03.01.b052 | Sync content with a connected device | Sync music, TV shows, podcasts, books, and more. | An iPhone, iPad, or iPod touch is connected to the Mac. |

#### Change how folders are displayed in the Finder on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchldaafb302/27/mac/27) · macOS 27 text present: no

Finder folders can be displayed in Icon, List, Column, or Gallery view. View options can be customized for one folder or applied as defaults to folders using the same view.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.03.01.e067 | Dock | other | At the bottom of the Mac screen | Click to open a Finder window. |
| MC.03.01.e068 | Finder window | window | Opened from the Dock | Displays the selected folder and its contents. |
| MC.03.01.e069 | Recents | other | Finder sidebar | An example of a standard folder. |
| MC.03.01.e070 | Desktop | other | Finder sidebar | An example of a standard folder. |
| MC.03.01.e071 | View buttons | button | Finder toolbar, or in a pop-up menu depending on window width | Choose the view used to display the selected folder. |
| MC.03.01.e072 | Icon | button | Finder toolbar or its pop-up menu | Selects Icon view. |
| MC.03.01.e073 | List | button | Finder toolbar or its pop-up menu | Selects List view. |
| MC.03.01.e074 | Column | button | Finder toolbar or its pop-up menu | Selects Column view. |
| MC.03.01.e075 | Gallery | button | Finder toolbar or its pop-up menu | Selects Gallery view. |
| MC.03.01.e076 | Icon View | view | Finder window | Displays items as icons arranged in a grid. |
| MC.03.01.e077 | List View | view | Finder window | Displays items in a list arranged by name, with other columns for additional information. |
| MC.03.01.e078 | Column View | view | Finder window | Displays items in columns. Selecting an item shows its thumbnail and details in the preview column. |
| MC.03.01.e079 | Gallery View | view | Finder window | Displays items in a row along the bottom, with a large preview above the row and details in the preview column. |
| MC.03.01.e080 | View | menu | Finder menu bar | Contains the Show View Options menu item. |
| MC.03.01.e081 | Show View Options | menu item | View menu in the Finder menu bar | Opens options for customizing the selected folder's view. |
| MC.03.01.e082 | Icon size | control | View options for Icon view | A slider used to change icon size. |
| MC.03.01.e083 | Column header | control | List View | Click to sort the list by that column; Control-click to choose columns to show or hide. |
| MC.03.01.e084 | Always open in | checkbox | View options for a folder | When selected, the folder always opens in the customized view. |
| MC.03.01.e085 | Browse in | checkbox | View options for a folder | When selected for a folder containing subfolders, those subfolders also open in that view. |
| MC.03.01.e086 | Use as Defaults | button | View options for a folder | Applies the settings to other folders that use the same view. Not available for Column view. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.03.01.b053 | Click Dock | A Finder window opens. | On a Mac. |
| MC.03.01.b054 | Select a folder and click a View button | The folder is displayed in the selected view: Icon, List, Column, or Gallery. | The View buttons may appear in a pop-up menu depending on window width. |
| MC.03.01.b055 | Make the Finder window wider | The View buttons appear in a row. | When the buttons are in a pop-up menu because of the window width. |
| MC.03.01.b056 | Use Icon view | Items appear as icons arranged in a grid. | — |
| MC.03.01.b057 | Choose View > Show View Options, then drag the “Icon size” slider | The icon size changes. | In Icon view. |
| MC.03.01.b058 | Use List view | Items appear in a list arranged by name, with other columns showing additional information. | — |
| MC.03.01.b059 | Click a column header | The list is sorted by that column. | In List view. |
| MC.03.01.b060 | Control-click a column header, then choose columns | Columns can be shown or hidden. | In List view. |
| MC.03.01.b061 | Select an item | Its thumbnail and details appear in the preview column. | In Column view. |
| MC.03.01.b062 | Swipe left or right on the trackpad, or press the Left or Right Arrow key | Browse along the row of items; the large preview and preview-column details update for the current item. | In Gallery view. |
| MC.03.01.b063 | Use Gallery view | Items appear in a row along the bottom of the Finder window, with a large preview above the row and details in the preview column. | — |
| MC.03.01.b064 | Use Column or Gallery view with images | You can show more details, add tags, or rotate images. | The available actions depend on the type of item. |
| MC.03.01.b065 | Click the additional-options control | Additional options are available. | The control's label or icon is not specified in the provided text. |
| MC.03.01.b066 | Select a folder and open View > Show View Options | View options for the selected folder are shown. | The options vary by view. |
| MC.03.01.b067 | Set view options | The selected folder's view can be customized; examples include adjusting icon size and grid spacing in Icon view, specifying how recent dates are shown in List view, and hiding filenames in Gallery view. | Available options depend on the view. |
| MC.03.01.b068 | Select “Always open in” | The folder always opens in the customized view. | — |
| MC.03.01.b069 | Select “Browse in” | Subfolders also open in the view. | The main folder contains subfolders. |
| MC.03.01.b070 | A subfolder opens in a different view | Select the subfolder, open View > Show View Options, and deselect “Always open in” and “Browse in” for that subfolder. The checkboxes should be selected for the main folder and deselected for the subfolder. | The main folder and subfolder have conflicting view checkbox settings. |
| MC.03.01.b071 | Click Use as Defaults | The settings are applied to any folder that uses the same view. | This option is unavailable for Column view. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.03.01.k004 | Command-J | Opens Show View Options. | After selecting a folder in Finder. |
| MC.03.01.k005 | Left Arrow | Browses left along the row of items. | In Gallery view. |
| MC.03.01.k006 | Right Arrow | Browses right along the row of items. | In Gallery view. |

#### Use tabs in windows on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchla4695cce/27/mac/27) · macOS 27 text present: no

Apple Support instructions for setting when documents open in tabs, adding and navigating tabs, merging windows, and closing tabs.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.03.01.e087 | System Settings | app | Referenced in the settings instructions | App where the Desktop & Dock settings are accessed. |
| MC.03.01.e088 | Desktop & Dock | setting | System Settings | Settings area containing the Windows section and tab preference. |
| MC.03.01.e089 | Open Desktop & Dock settings for me | other | Below the instruction to click Desktop & Dock | Link offered to open Desktop & Dock settings. |
| MC.03.01.e090 | Windows | view | Desktop & Dock settings | Section containing the tab preference. |
| MC.03.01.e091 | Prefer tabs when opening documents | control | Desktop & Dock settings > Windows | Pop-up menu for choosing when documents open in tabs. |
| MC.03.01.e092 | Always | menu item | Prefer tabs when opening documents pop-up menu | One of the available options. |
| MC.03.01.e093 | In Full Screen | menu item | Prefer tabs when opening documents pop-up menu | One of the available options. |
| MC.03.01.e094 | Never | menu item | Prefer tabs when opening documents pop-up menu | One of the available options. |
| MC.03.01.e095 | TextEdit | app | Example in the instructions for opening a document | Example app in which to open a document. |
| MC.03.01.e096 | View | menu | App menu bar | Contains tab display and overview commands. |
| MC.03.01.e097 | Show Tab Bar | menu item | View menu | Shows the tab bar when it is not displayed. |
| MC.03.01.e098 | Show All Tabs | menu item | View menu | Shows thumbnails of all open tabs in a window. |
| MC.03.01.e099 | Exit Tab Overview | menu item | View menu | Returns from tab overview to the current tab. |
| MC.03.01.e100 | Window | menu | App menu bar | Contains commands for moving a tab to a new window and merging windows. |
| MC.03.01.e101 | Move Tab to New Window | menu item | Window menu | Moves the selected tab into a new window. |
| MC.03.01.e102 | Merge All Windows | menu item | Window menu | Merges eligible windows of the current type into one tabbed window. |
| MC.03.01.e103 | tab | control | App window | A document or item shown as a tab; its name appears after the document is saved. |
| MC.03.01.e104 | tab overview | view | App window | Displays thumbnails of all open tabs. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.03.01.b072 | Choose an option from Prefer tabs when opening documents | Documents open in tabs or separate windows according to the selected option. | In apps that support tabs; options are Always, In Full Screen, and Never. |
| MC.03.01.b073 | Press Option-Command-N | Opens a new window in an app. | Works even when Prefer tabs when opening documents is set to Always. |
| MC.03.01.b074 | Open a document and click the tab control | Adds a tab for the document. | In an app that supports tabs; the article does not specify the control's label. |
| MC.03.01.b075 | Choose View > Show Tab Bar | Shows the tab bar. | When the tab bar is not shown. |
| MC.03.01.b076 | Save the document | The document's name appears as the tab name. | After opening a document in a tab. |
| MC.03.01.b077 | Press Command-N | Adds a tab. | In some apps, when Prefer tabs when opening documents is set to In Full Screen or Always. |
| MC.03.01.b078 | Press Option-Command-N | Adds a tab. | In some apps, when Prefer tabs when opening documents is set to Never. |
| MC.03.01.b079 | Choose View > Show All Tabs | Shows thumbnails of all open tabs in a window. | In an app that has tabs. |
| MC.03.01.b080 | Click a tab thumbnail in tab overview | Goes to that tab. | While tab overview is displayed. |
| MC.03.01.b081 | Choose View > Exit Tab Overview | Returns to the current tab. | While in tab overview. |
| MC.03.01.b082 | Click a tab | Moves to that tab. | In an app that has tabs. |
| MC.03.01.b083 | Drag a tab left or right | Reorders the tabs. | In an app window with tabs. |
| MC.03.01.b084 | Choose Window > Move Tab to New Window | Makes the selected tab a separate window. | After selecting the tab. |
| MC.03.01.b085 | Drag a tab out of its window | Makes the tab a separate window. | In an app window with tabs. |
| MC.03.01.b086 | Choose Window > Merge All Windows | Merges windows into one tabbed window. | In an app with more than one window; if it has different types of windows, only the type currently being worked in is merged. |
| MC.03.01.b087 | Move the pointer over a tab and click the close control | Closes the tab. | In an app that has tabs; the close control is not given a textual label in the article. |
| MC.03.01.b088 | Move the pointer over the tab to keep open, then Option-click the close control | Closes all other tabs. | In an app that has tabs. |
| MC.03.01.b089 | Press Command-H | Hides the active app. | When the app is active. |
| MC.03.01.b090 | Press Command-Q | Quits the active app. | When the app is active. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.03.01.k007 | Option-Command-N | Open a new window. | Can be used even when the tab preference is Always. |
| MC.03.01.k008 | Command-N | Add a tab. | In some apps, when the tab preference is In Full Screen or Always. |
| MC.03.01.k009 | Option-Command-N | Add a tab. | In some apps, when the tab preference is Never. |
| MC.03.01.k010 | Control-Tab | Go to the next tab. | In an app that has tabs. |
| MC.03.01.k011 | Control-Shift-Tab | Go to the previous tab. | In an app that has tabs. |
| MC.03.01.k012 | Option-click | Close all other tabs. | Move the pointer over the tab to keep open, then Option-click its close control. |
| MC.03.01.k013 | Command-H | Hide the active app. | When the app is active. |
| MC.03.01.k014 | Command-Q | Quit the active app. | When the app is active. |

System Settings paths referenced: System Settings > Desktop & Dock > Windows > Prefer tabs when opening documents

#### Independent reports (MC.03.01)

| ID | Kind | Area | Claim | Source |
|---|---|---|---|---|
| MC.03.01.x01 | behavior | Finder | In the first developer beta of macOS 27, icons remain in Finder menus where they help distinguish between display variants. (translated) | T4 [iFun](https://www.ifun.de/finder-menues-in-macos-27-wieder-ohne-mini-symbole-281385/) |
| MC.03.01.x02 | behavior | Finder | The comments mention “Seitenleiste” (Sidebar) and “Vorschau” (Preview) as Finder menu items whose icons can be helpful, and “Symbolleiste” (Toolbar) and “Darstellungsoptionen” (View Options) as examples of more generic icons. (translated) | T4 [iFun](https://www.ifun.de/finder-menues-in-macos-27-wieder-ohne-mini-symbole-281385/) |
| MC.03.01.x03 | measurement | Finder | Browsing folders on an Ethernet-connected Mac mini server felt nearly as fast as browsing local folders. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/10/) |
| MC.03.01.x04 | measurement | Finder | Listing the contents of large folders on networked storage was smooth and fast during the summer. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/10/) |
| MC.03.01.x05 | changed | Finder | Finder’s icon has a rounder nose. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/2/) |
| MC.03.01.x06 | behavior | other | Golden Gate does not remove the icon rule introduced in macOS 26: icon glyphs must fit within their background tile. Icons that do not fit can appear shrunken inside a generic “squircle jail” tile. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/2/) |
| MC.03.01.x07 | new | other | The article identifies a new Siri icon and redesigned icons for Preview, Chess, Finder, Image Playground, and iPhone Mirroring. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/2/) |
| MC.03.01.x08 | new | Finder | Finder can scan a file’s contents and offer filename suggestions. | T4 [Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/) |
| MC.03.01.x09 | new | Finder | Siri can suggest file names based on file content when a file name is highlighted, including in save dialogs; the article says suggestions are currently file by file. | T4 [Pizazoo](https://pizazoo.com/blog/macos-27-screenshot-features/) |
| MC.03.01.x10 | new | Finder | Finder adds autosize and reset controls for columns. Right-clicking or Control-clicking the bottom of a column reveals options for adjusting the view. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MC.03.01.x11 | new | Finder | Apple Intelligence can suggest a name for a file or folder based on its contents in Finder and other Apple apps. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MC.03.01.x12 | changed | Finder | In macOS Golden Gate, the Finder window sidebar extends to the edge of the frame, giving windows a flatter appearance. | T4 [AppleInsider](https://appleinsider.com/articles/26/06/09/dont-get-excited-for-big-liquid-glass-changes-in-macos-27-because-they-arent-there) |
| MC.03.01.x13 | behavior | Finder | In macOS Tahoe, the Finder sidebar was inset from the window edge and slightly raised. | T4 [AppleInsider](https://appleinsider.com/articles/26/06/09/dont-get-excited-for-big-liquid-glass-changes-in-macos-27-because-they-arent-there) |
| MC.03.01.x14 | new | Finder | The post reports a new Finder option to get suggested file names. | T5 [MacRumors Forums](https://forums.macrumors.com/threads/macos-27-all-the-little-things.2483520/) |
| MC.03.01.x15 | new | Finder | The post reports a new iCloud icon in Finder’s Customize Toolbar. | T5 [MacRumors Forums](https://forums.macrumors.com/threads/macos-27-all-the-little-things.2483520/) |
| MC.03.01.x16 | new | Finder | Finder now supports pasting items from the clipboard to the file system. | T5 [MacRumors Forums](https://forums.macrumors.com/threads/macos-27-all-the-little-things.2483520/) |
| MC.03.01.x17 | behavior | Finder | The post reports that Option-click can be used for secondary sorting. | T5 [MacRumors Forums](https://forums.macrumors.com/threads/macos-27-all-the-little-things.2483520/) |
| MC.03.01.x18 | bug | Finder | Fixed an issue where suggested names for files or folders could be more generic than their contents. | T4 [BetaProfile](https://betaprofile.dev/macos-27-rc/) |
| MC.03.01.x19 | new | Finder | Finder can intelligently suggest file and folder names based on their contents. | T4 [9to5Mac](https://9to5mac.com/2026/09/14/macos-27-golden-gate-now-available-here-is-everything-new/) |

### MC.03.02 Sidebar

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).


#### Customize the Finder sidebar on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchl83c9e8b8/27/mac/27) · macOS 27 text present: no

The page explains how to show, hide, resize, and rearrange the Finder sidebar; choose which items appear; and add or remove sidebar links.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.03.02.e001 | Dock | other | Mac interface | Clicking it is given as the way to open a Finder window. |
| MC.03.02.e002 | View | menu | Menu bar | Contains the Hide Sidebar and Show Sidebar commands. |
| MC.03.02.e003 | Hide Sidebar | menu item | View menu | Hides the Finder sidebar. |
| MC.03.02.e004 | Show Sidebar | menu item | View menu | Shows the Finder sidebar. |
| MC.03.02.e005 | Show Toolbar | menu item | View menu | The page says to choose this if Show Sidebar is dimmed. |
| MC.03.02.e006 | Finder | menu | Menu bar | Contains the Settings command. |
| MC.03.02.e007 | Settings | menu item | Finder menu | Opens Finder settings. |
| MC.03.02.e008 | Sidebar | panel | Finder > Settings | Contains selectable items for the Finder sidebar. |
| MC.03.02.e009 | Favorites | view | Finder sidebar | Section where folders, disks, files, and apps can be added; its visibility can be changed. |
| MC.03.02.e010 | Customize the Finder toolbar on Mac | other | See also | Related support-page link. |
| MC.03.02.e011 | Ways to quickly see items in the Finder on Mac | other | Related links | Related support-page link. |
| MC.03.02.e012 | Mac User Guide | other | Related links | Guide link. |
| MC.03.02.e013 | Customize the Finder sidebar on Mac | other | Related links | Support-page link. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.03.02.b001 | Click Dock to open a Finder window. | A Finder window opens. | On your Mac. |
| MC.03.02.b002 | Choose View > Hide Sidebar. | The Finder sidebar is hidden. | In a Finder window. |
| MC.03.02.b003 | Choose View > Show Sidebar. | The Finder sidebar is shown. | In a Finder window; Show Sidebar may be dimmed. |
| MC.03.02.b004 | If Show Sidebar is dimmed, choose View > Show Toolbar. | The toolbar is shown, allowing the sidebar command to be used. | In a Finder window when Show Sidebar is dimmed. |
| MC.03.02.b005 | Hold the pointer over the divider between the sidebar and the window, then drag the right side of the divider right or left. | The sidebar is resized. | In a Finder window. |
| MC.03.02.b006 | Choose Finder > Settings, click Sidebar, then select or deselect items. | The items shown in the sidebar are changed. | In Finder. |
| MC.03.02.b007 | Drag a sidebar item to a new location. | The item is rearranged in the sidebar. | In the Finder sidebar. |
| MC.03.02.b008 | Position the pointer over a sidebar section heading, then click the show control. | All items in that section are shown. | In the Finder sidebar; the source does not render the control's icon. |
| MC.03.02.b009 | Position the pointer over a sidebar section heading, then click the hide control. | All items in that section are hidden. | In the Finder sidebar; the source does not render the control's icon. |
| MC.03.02.b010 | Position the pointer over Favorites and click its hide control. | Favorites are temporarily hidden. | In the Finder sidebar; the source does not render the control's icon. |
| MC.03.02.b011 | Drag a folder or disk to Favorites. | The folder or disk is added to the sidebar as an alias; its original location is unchanged. | If Favorites is not visible, choose Finder > Settings, click Sidebar, and select at least one item in the Favorites section. |
| MC.03.02.b012 | Press and hold Command while dragging a file to Favorites. | The file is added to the sidebar as an alias; its original location is unchanged. | If Favorites is not visible, choose Finder > Settings > Sidebar and select at least one item in the Favorites section. |
| MC.03.02.b013 | Press and hold Command while dragging an app icon to Favorites. | The app is added to Favorites in the sidebar. | In Finder. |
| MC.03.02.b014 | Drag an item's icon out of the sidebar until the remove indicator appears. | The sidebar link disappears; the folder, disk, or file remains in its original location on the Mac. | For an item in the Finder sidebar. |
| MC.03.02.b015 | Choose Finder > Settings. | Finder settings can be changed. | The page refers readers to this for other Finder settings. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.03.02.k001 | Command (held) | Hold the key while dragging a file or app icon to Favorites to add it to the sidebar. | Finder sidebar. |

### MC.03.03 Tags

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).

Also covered by: [Change Finder settings on Mac](https://support.apple.com/guide/mac-help/mchlp2803/27/mac/27)


#### Use tags to organize files on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp15236/27/mac/27) · macOS 27 text present: no

Tags can be added to files and folders, used to find and organize items, edited in Finder settings, and removed from items or from the Mac.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.03.03.e001 | File | menu | Menu bar | Menu used to access Save and tagging options. |
| MC.03.03.e002 | Save | menu item | File menu | Opens the Save dialog for a new file. |
| MC.03.03.e003 | document title | view | Open document | The title area; the instructions say to hold the pointer to its right. |
| MC.03.03.e004 | Tags | control | Open document and Save dialog | The Tags field is used to enter a new tag or choose one from the list. |
| MC.03.03.e005 | Tags | menu item | File menu or Control-click menu for an item | Opens options to choose or enter tags. |
| MC.03.03.e006 | Dock | other | Mac desktop | Clicking Dock opens a Finder window, according to the instructions. |
| MC.03.03.e007 | Finder | app | Mac | Used to search for, view, and manage tagged items. |
| MC.03.03.e008 | search field | control | Finder window | Enter a tag color or name to search. |
| MC.03.03.e009 | suggestions | view | Finder search | Suggestions include tags matching the entered color or name. |
| MC.03.03.e010 | sidebar | view | Finder window | Can display tags; clicking a tag shows items with that tag. |
| MC.03.03.e011 | Settings | menu item | Finder menu | Opens Finder settings. |
| MC.03.03.e012 | Tags | panel | Finder Settings | Contains controls for choosing sidebar tags and managing tags. |
| MC.03.03.e013 | View | menu | Menu bar | Contains Show View Options. |
| MC.03.03.e014 | Show View Options | menu item | View menu | Opens view options. |
| MC.03.03.e015 | Sort By | control | View Options | Pop-up menu used to choose Tags as the sort criterion. |
| MC.03.03.e016 | Tags | menu item | Sort By pop-up menu | Sort criterion for items. |
| MC.03.03.e017 | List view | view | Finder window | A Finder view in which a Tags column can be displayed. |
| MC.03.03.e018 | Tags | column | Finder List view | Can be displayed and clicked to sort items; clicking the column name again reverses the sort order. |
| MC.03.03.e019 | Edit Tags | control | Finder window or desktop item actions | Opens tag selection for removing tags from an item. |
| MC.03.03.e020 | Delete | key | After selecting tags to remove | Removes the selected tags from an item. |
| MC.03.03.e021 | blue checkbox | control | Finder Tags settings, to the right of a tag | Controls whether a tag appears in the Finder sidebar. |
| MC.03.03.e022 | Favorite Tags | view | Bottom of Finder Tags settings | Section containing tags in the shortcut menu; up to seven tags can be listed. |
| MC.03.03.e023 | Delete Tag | button | Finder Tags settings | Deletes a selected tag after confirmation by clicking Delete Tag. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.03.03.b001 | Hold the pointer to the right of an open document's title, click the adjacent control, then click the Tags field. | Enter a new tag or choose one from the list to tag the open file. | An open file. |
| MC.03.03.b002 | Choose File > Save, then click the Tags field in the Save dialog. | Enter a new tag or choose one from the list to tag the new file. | Saving a new file. |
| MC.03.03.b003 | Select an item on the desktop or in Finder, then open the File menu or Control-click the item. | Show tagging options. | A file or folder is selected. |
| MC.03.03.b004 | Choose a color above Tags in the item's menu. | Apply the tag represented by that color; the tag name replaces “Tags” as the pointer moves over the color. | The File menu or Control-click menu is open for a selected item. |
| MC.03.03.b005 | Click Tags in the item's menu. | Choose from more tags or enter a new tag. | The File menu or Control-click menu is open for a selected item. |
| MC.03.03.b006 | Select a file and press Control-1 through Control-7. | Add or remove the corresponding favorite tag. | The tag is among the favorite tags assigned to the Control-1 through Control-7 shortcuts. |
| MC.03.03.b007 | Select a file and press Control-0 (zero). | Remove all tags from the file. | A file is selected. |
| MC.03.03.b008 | Click Dock to open a Finder window, enter a tag color or name in the search field, then select the tag from the suggestions. | Search for items with that tag. | In Finder. |
| MC.03.03.b009 | Click a tag in the Finder sidebar. | See everything with that tag. | The tag is shown in the Finder sidebar. |
| MC.03.03.b010 | Choose Finder > Settings, click Tags, and select the tags to show. | Change which tags appear in the Finder sidebar. | In Finder. |
| MC.03.03.b011 | Choose View > Show View Options, click Sort By, and choose Tags. | Sort items by tag. | In any Finder view. |
| MC.03.03.b012 | In List view, Control-click a column and select Tags. | Display the Tags column. | In Finder List view. |
| MC.03.03.b013 | Move the pointer over the Tags column and click it. | Sort items by the Tags column; click the column name again to reverse the sort order. | The Tags column is displayed in List view. |
| MC.03.03.b014 | Select an item in a Finder window or on the desktop, click Edit Tags, select tags, then press Delete. | Remove the selected tags from the item. | A file or folder is selected. |
| MC.03.03.b015 | Choose Finder > Settings, click Tags, select tags to remove, click the adjacent control, then click Delete Tag. | Remove the selected tags from the Mac. | In Finder. |
| MC.03.03.b016 | Select the blue checkbox to the right of a tag. | Show or hide that tag in the Finder sidebar. | In Finder Tags settings. |
| MC.03.03.b017 | Click the color next to a tag and choose a new color. | Change the tag's color. | In Finder Tags settings. |
| MC.03.03.b018 | Click a tag, click the tag's name, and enter a new name. | Change the tag's name. | In Finder Tags settings. |
| MC.03.03.b019 | Click the adjacent control for creating a tag. | Create a new tag. | In Finder Tags settings. |
| MC.03.03.b020 | Select a tag, click the adjacent control, then click Delete Tag. | Delete the tag. | In Finder Tags settings. |
| MC.03.03.b021 | Select a tag in the list and drag it over the tag to replace in Favorite Tags. | Add the selected tag to the shortcut menu in place of the tag it replaces. | In Finder Tags settings; the shortcut menu can contain up to seven tags. |
| MC.03.03.b022 | Drag a tag out of Favorite Tags until the removal indicator appears. | Remove the tag from the shortcut menu. | In Finder Tags settings. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.03.03.k001 | Control-1 through Control-7 | Add or remove the corresponding favorite tag. | Select a file; shortcuts correspond to favorite tags. |
| MC.03.03.k002 | Control-0 (zero) | Remove all tags from a file. | Select a file. |
| MC.03.03.k003 | Delete | Remove selected tags from an item. | After selecting tags in Edit Tags. |
| MC.03.03.k004 | Control-click | Open the item's contextual menu, including tag options. | A file or folder on the desktop or in Finder. |

System Settings paths referenced: Finder > Settings > Tags

### MC.03.04 Search & smart folders

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).

Also covered by: [Use the Finder on Mac](https://support.apple.com/guide/mac-help/mchlp2605/27/mac/27)


#### Create or change a Smart Folder on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp2804/27/mac/27) · macOS 27 text present: no

A Smart Folder lists files that match specified criteria and updates automatically as matching files are added, changed, or deleted. The page describes creating one in Finder and gives limited instructions for changing its criteria.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.03.04.e001 | Finder | other | App used to create or change a Smart Folder | The instructions say to perform these tasks in Finder. |
| MC.03.04.e002 | Smart Folder | other | Finder | A folder that lists files matching specified criteria. |
| MC.03.04.e003 | search field | control | Finder search interface | Enter a topic, phrase, date, or another characteristic of files to set the first rule. |
| MC.03.04.e004 | suggestions | view | Below the search field | Choose from suggestions that appear after entering a search term. |
| MC.03.04.e005 | Action | menu | In the search field | An Action pop-up menu used to refine a search. |
| MC.03.04.e006 | Save | button | Smart Folder creation flow | Click to proceed to naming and choosing a location, and click again to save the Smart Folder. |
| MC.03.04.e007 | Add To Sidebar | control | Smart Folder save flow | Deselect this option if you don’t want the Smart Folder in the sidebar. |
| MC.03.04.e008 | Find what you need with Spotlight on Mac | other | See also | A linked support-page title. |
| MC.03.04.e009 | Organize files in folders on Mac | other | See also | A linked support-page title. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.03.04.b001 | Add, change, or delete files on the Mac that meet the Smart Folder’s criteria. | The list of files in the Smart Folder is updated automatically. | The files meet the Smart Folder’s criteria. |
| MC.03.04.b002 | Enter a topic, phrase, date, or another file characteristic in the search field. | Suggestions appear below the search field; choose from them to set the first rule. | While specifying criteria in Finder. |
| MC.03.04.b003 | Click the Action pop-up menu in the search field. | Refine the search. | While specifying criteria in Finder. |
| MC.03.04.b004 | Click Save after specifying criteria. | Enter a name and location for the Smart Folder. | During Smart Folder creation. |
| MC.03.04.b005 | Deselect Add To Sidebar. | The Smart Folder is not placed in the sidebar. | While saving the Smart Folder. |
| MC.03.04.b006 | Click Save after entering a name and location. | Save the Smart Folder. | During Smart Folder creation. |
| MC.03.04.b007 | Use certain characters, including punctuation such as a colon (:), in a Smart Folder name. | The name cannot be used; if an error message appears, try another name. | When naming a Smart Folder. |

### MC.03.05 Quick Look

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).


#### View and edit files with Quick Look on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mh14119/27/mac/27) · macOS 27 text present: no

Quick Look previews files without opening them and provides controls for browsing, editing, sharing, and viewing items.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.03.05.e001 | Quick Look window | window | Opens after selecting one or more items and pressing the Space bar | Shows a full-size preview. When multiple items are selected, the last item selected is shown first. |
| MC.03.05.e002 | Space bar | control | Keyboard | Opens or closes Quick Look for the selected item or items. |
| MC.03.05.e003 | Window corners | other | Corners of the Quick Look window | Can be dragged to resize the window. |
| MC.03.05.e004 | Unnamed resize button | button | Quick Look window | The page says to click an icon to resize the window, but does not provide its label or icon. |
| MC.03.05.e005 | Unnamed rotate button | button | Quick Look window | The page says to click an icon to rotate the item, but does not provide its label or icon. |
| MC.03.05.e006 | Unnamed Markup button | button | Quick Look window | The page says to click an icon to mark up an item, but does not provide its label or icon. |
| MC.03.05.e007 | Unnamed trim button | button | Quick Look window | The page says to click an icon to trim an audio or video item, but does not provide its label or icon. |
| MC.03.05.e008 | Trimming bar | control | Quick Look window when trimming audio or video | Contains yellow handles for setting the trim. |
| MC.03.05.e009 | Yellow handles | control | Trimming bar | Drag to set the trim for an audio or video item. |
| MC.03.05.e010 | Unnamed play button | button | Quick Look window when trimming audio or video | The page says to click an icon to test trim changes, but does not provide its label or icon. |
| MC.03.05.e011 | Revert | button | Quick Look window when trimming audio or video | Starts the trim over. |
| MC.03.05.e012 | Done | button | Quick Look window when trimming audio or video | Completes the trim and presents choices to replace the original file or create a new one. |
| MC.03.05.e013 | Unnamed browsing arrows | control | Near the top-left of the Quick Look window | Browse between selected items; the page does not provide labels for the arrows. |
| MC.03.05.e014 | Unnamed full-screen browsing control | control | Quick Look in full screen | The page says to click a control to browse items but does not provide its label or icon. |
| MC.03.05.e015 | Unnamed grid button | button | Quick Look window when multiple items are selected | Shows items in a grid; the page does not provide its label or icon. |
| MC.03.05.e016 | Open with [App] | button | Quick Look window | Opens the item in the named app. |
| MC.03.05.e017 | Unnamed Share button | button | Quick Look window | The page says to click an icon to share an item, but does not provide its label or icon. |
| MC.03.05.e018 | Copy Subject | menu item | Context menu shown by Control-clicking a screenshot or photo | Copies the picture subject isolated from its background. |
| MC.03.05.e019 | Live Photo | control | Bottom-left corner of a Live Photo in Quick Look | Click to play the Live Photo's video portion again. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.03.05.b001 | Select one or more items and press the Space bar | A Quick Look window opens. If multiple items were selected, the last item selected is shown first. | Items can be in Finder windows, on the desktop, in emails and messages, or in other places. |
| MC.03.05.b002 | Drag the corners of the Quick Look window | Resize the window. | While the Quick Look window is open. |
| MC.03.05.b003 | Click the unnamed resize button | Resize the window. | While the Quick Look window is open. |
| MC.03.05.b004 | Press Command-Plus (+) | Make the image bigger. | While viewing an item in Quick Look. |
| MC.03.05.b005 | Press Command-Minus (–) | Make the image smaller. | While viewing an item in Quick Look. |
| MC.03.05.b006 | Click the unnamed rotate button | Rotate the item. Clicking again continues rotating it. | While viewing an item in Quick Look. |
| MC.03.05.b007 | Click the unnamed Markup button | Mark up the item. | While viewing an item in Quick Look. |
| MC.03.05.b008 | Click the unnamed trim button, then drag the yellow handles in the trimming bar | Trim the item. | For an audio or video item. |
| MC.03.05.b009 | Click the unnamed play button | Test the trim changes. | While trimming an audio or video item. |
| MC.03.05.b010 | Click Revert | Start the trim over. | While trimming an audio or video item. |
| MC.03.05.b011 | Click Done | Finish the trim and choose whether to replace the original file or create a new one. | While trimming an audio or video item. |
| MC.03.05.b012 | Click the browsing arrows near the top-left of the window, or press Left Arrow or Right Arrow | Browse between the selected items. | When multiple items are selected. |
| MC.03.05.b013 | Click the unnamed full-screen browsing control | Browse between items. | When viewing in full screen; the page does not identify the control. |
| MC.03.05.b014 | Click the unnamed grid button | Show the items in a grid. | When multiple items are selected. |
| MC.03.05.b015 | Click Open with [App] | Open the item in the named app. | While viewing an item in Quick Look. |
| MC.03.05.b016 | Click the unnamed Share button and choose how to share the item | Share the item using the chosen method. | While viewing an item in Quick Look. |
| MC.03.05.b017 | Control-click the image, then choose Copy Subject | Copy the subject isolated from the background; it can then be pasted into a document, email, text message, or note. | The item must be a screenshot or photo. |
| MC.03.05.b018 | Press the Space bar or click the unnamed close control | Close Quick Look. | When finished viewing. |
| MC.03.05.b019 | View an HDR image in Quick Look | The image can be viewed as HDR, showing a wider range of brightness levels and colors. | When the same HDR image is also viewed in another app, Quick Look shows it as HDR only while Quick Look is in the foreground. |
| MC.03.05.b020 | Open a Live Photo in the Quick Look window | The video portion plays automatically. | The item is a Live Photo. |
| MC.03.05.b021 | Click Live Photo | Play the Live Photo's video portion again. | The Live Photo is open in Quick Look. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.03.05.k001 | Space bar | Open Quick Look for selected items; press again to close it. | Items selected on the Mac. |
| MC.03.05.k002 | Command-Plus (+) | Make the image bigger. | Quick Look. |
| MC.03.05.k003 | Command-Minus (–) | Make the image smaller. | Quick Look. |
| MC.03.05.k004 | Left Arrow | Browse to another selected item. | When multiple items are selected in Quick Look. |
| MC.03.05.k005 | Right Arrow | Browse to another selected item. | When multiple items are selected in Quick Look. |
| MC.03.05.k006 | Control-click | Open the context menu used to choose Copy Subject. | On a screenshot or photo in Quick Look. |

### MC.03.06 Get Info

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).


#### Get file, folder, and disk information on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp1774/27/mac/27) · macOS 27 text present: no

Finder provides information about files, folders, and disks through the status bar, Get Info windows, the path bar, item info below icons, and the Preview pane.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.03.06.e001 | Finder | other | App | The app used for the file, folder, and disk information actions described. |
| MC.03.06.e002 | status bar | view | Bottom or top of the Finder window | Displays the number of items in the disk or folder and the amount of available disk space. |
| MC.03.06.e003 | View | menu | Finder menu bar | Contains the Show Status Bar, Show Path Bar, Show View Options, and Show Preview commands. |
| MC.03.06.e004 | Show Status Bar | menu item | View menu | Shows the status bar in the Finder window. |
| MC.03.06.e005 | File | menu | Finder menu bar | Contains Get Info and Get Summary Info. |
| MC.03.06.e006 | Get Info | menu item | File menu | Opens an information window for the selected item. |
| MC.03.06.e007 | Get Summary Info | menu item | File menu | Opens information windows for selected multiple items. |
| MC.03.06.e008 | information window | window | For the selected item | Displays information about the item. |
| MC.03.06.e009 | Dock | other | Mac desktop | The page says to click Dock to open a Finder window. |
| MC.03.06.e010 | Show Path Bar | menu item | View menu | Shows the path bar in the Finder window. |
| MC.03.06.e011 | path bar | view | Near the bottom of the Finder window | Displays the location and nested folders that contain the file or folder. |
| MC.03.06.e012 | Copy “folder” as Pathname | menu item | Context menu for a folder in the path bar | Copies the folder’s pathname. The word “folder” represents the folder name. |
| MC.03.06.e013 | Show View Options | menu item | View menu | Opens the view options for the folder or desktop. |
| MC.03.06.e014 | Show item info | control | View Options | A selectable option for showing information below icons; available only for Icon view. |
| MC.03.06.e015 | Icon view | view | Finder | The view mode in which Show item info is available. |
| MC.03.06.e016 | Show Preview | menu item | View menu | Shows the Preview pane. |
| MC.03.06.e017 | Preview pane | panel | Finder window | The page identifies this as a place to show information. |
| MC.03.06.e018 | iCloud Status column | view | Finder | May show status indicators for folders and files when iCloud Drive is used to store documents on the Mac. |
| MC.03.06.e019 | Check your iCloud Drive file and folder status | other | See also | Linked reference. |
| MC.03.06.e020 | Manage iCloud storage on Mac | other | See also | Linked reference. |
| MC.03.06.e021 | View and edit files with Quick Look on Mac | other | See also | Linked reference. |
| MC.03.06.e022 | Sort and arrange items in the Finder on Mac | other | See also | Linked reference. |
| MC.03.06.e023 | See the devices connected to your Mac | other | See also | Linked reference. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.03.06.b001 | Choose View > Show Status Bar. | The Finder window displays the status bar. | When the status bar is not visible. |
| MC.03.06.b002 | Select an item in a Finder window or on the desktop, then choose File > Get Info or press Command-I. | An information window opens for the item. | The selected item can be a file, folder, or disk. |
| MC.03.06.b003 | Select multiple items, hold the Control key, then choose File > Get Summary Info. | A separate information window opens for each item. | Multiple items are selected. |
| MC.03.06.b004 | Choose View > Show Path Bar. | The path bar displays the location and nested folders that contain the file or folder. | In a Finder window. |
| MC.03.06.b005 | Click the folder in the path bar. | The folder’s subfolders are shown in the Finder window. | The folder is represented in the path bar. |
| MC.03.06.b006 | Control-click the folder in the path bar, then choose Copy “folder” as Pathname. | The folder’s pathname is copied. | The folder is represented in the path bar. |
| MC.03.06.b007 | Open a folder in Finder or click the desktop, choose View > Show View Options, then select “Show item info.” | Information is shown below icons. | Only available for Icon view. Item info can include file size, movie length, image dimensions, and more. |
| MC.03.06.b008 | Open a folder in Finder, then choose View > Show Preview. | The Preview pane is shown. | In Finder. |
| MC.03.06.b009 | Use iCloud Drive to store documents on the Mac. | Folders and files may appear with a status indicator in the iCloud Status column. | The documents are stored on the Mac using iCloud Drive. |
| MC.03.06.b010 | Click Dock. | A Finder window opens. | As stated in the path-bar instructions. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.03.06.k001 | Command-I | Open an information window for the selected item. | A Finder window or the desktop, with an item selected. |
| MC.03.06.k002 | Option | Show the path bar momentarily. | In Finder. |
| MC.03.06.k003 | Control-click | Open the folder’s context menu in the path bar. | On a folder in the path bar. |

#### Change permissions for files, folders, or disks on Mac - Apple Support

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp1203/27/mac/27) · macOS 27 text present: no

The article explains how to view and change permissions for files, folders, and disks in Finder, manage users and owners, apply permissions to items in a folder or disk, and repair broken permissions in iCloud Drive.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.03.06.e024 | File | menu | Finder menu bar | Menu used with Get Info. |
| MC.03.06.e025 | Get Info | menu item | File menu | Opens the selected item's Info window. |
| MC.03.06.e026 | Info window | window | Finder | Contains the Sharing & Permissions section for the selected item. |
| MC.03.06.e027 | Sharing & Permissions | section | Bottom of the Info window | Section for viewing and changing permissions, users, groups, and ownership. |
| MC.03.06.e028 | Name | column | Sharing & Permissions section | Lists users and groups. |
| MC.03.06.e029 | Read & Write | menu item | Privilege pop-up menu in Sharing & Permissions | Allows a user to open the item and change it. |
| MC.03.06.e030 | Read only | menu item | Privilege pop-up menu in Sharing & Permissions | Allows a user to open the item, but not change its contents. |
| MC.03.06.e031 | Write only (Drop Box) | menu item | Privilege pop-up menu in Sharing & Permissions | Makes a folder into a drop box; users can copy items to it but can’t open it. Only the owner can open it. |
| MC.03.06.e032 | No Access | menu item | Privilege pop-up menu in Sharing & Permissions | Blocks all access to the item. |
| MC.03.06.e033 | user or group | other | Name column in Sharing & Permissions | A listed user or group can be selected. |
| MC.03.06.e034 | pop-up menu | control | Sharing & Permissions section | Used to choose a privilege setting for a selected user or group. |
| MC.03.06.e035 | Dock | other | Mac | The article says to click Dock to open a Finder window. |
| MC.03.06.e036 | Finder | app | Mac | Used to select items and open iCloud Drive. |
| MC.03.06.e037 | iCloud Drive | sidebar item | Finder sidebar | Location for files and folders that may have broken permissions. |
| MC.03.06.e038 | Repair | button | Error message at the top of the iCloud Drive folder | Starts repair of files or folders with permission problems. |
| MC.03.06.e039 | Sharing & Permissions | section | Info window | The article refers to controls for undoing changes, adding or removing users or groups, and changing an owner, but their labels are missing from the supplied text. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.03.06.b011 | Select a disk, folder, or file, then choose File > Get Info. | The Info window for the selected item opens. | On Mac, in Finder. |
| MC.03.06.b012 | If Sharing & Permissions information is not visible, click the control indicated in the article. | The section’s information becomes visible. | The article does not provide a label for the control; it notes that scrolling down may be necessary. |
| MC.03.06.b013 | Select a user or group in the Name column, click the pop-up menu, then choose Read & Write. | That user or group can open the item and change it. | In Sharing & Permissions. |
| MC.03.06.b014 | Select a user or group in the Name column, click the pop-up menu, then choose Read only. | That user or group can open the item, but cannot change its contents. | In Sharing & Permissions. |
| MC.03.06.b015 | Select a user or group in the Name column, click the pop-up menu, then choose Write only (Drop Box). | The folder becomes a drop box. Users can copy items to it but cannot open it; only its owner can open it. | The selected item must be a folder. |
| MC.03.06.b016 | Select a user or group in the Name column, click the pop-up menu, then choose No Access. | Access to the item is blocked for that user or group. | In Sharing & Permissions. |
| MC.03.06.b017 | Make changes to privilege settings, then use the undo control before closing the Info window. | Privilege-setting changes made since opening the Info window can be undone. | The undo control’s label is missing from the supplied text. |
| MC.03.06.b018 | Select a folder or disk, then choose File > Get Info and reveal Sharing & Permissions. | The article provides a way to apply permissions to all items in the folder or disk. | The supplied text omits the labels and details of the controls used to perform this action. |
| MC.03.06.b019 | Select an item, choose File > Get Info, and reveal Sharing & Permissions. | The item’s owner can be changed using the Name column and a control associated with the owner. | If the new owner is not listed, the article says to use an omitted control to add or select them. The supplied text truncates the owner-selection instruction. |
| MC.03.06.b020 | Make owner changes, then use the undo control before closing the Info window. | Owner changes made since opening the Info window can be undone. | The undo control’s label is missing from the supplied text. |
| MC.03.06.b021 | In Sharing & Permissions, click the add control. | A user or group can be added to the Name column. | The supplied text does not include the control’s label. |
| MC.03.06.b022 | Select a user or group in Sharing & Permissions, then click the remove control. | The selected user or group is removed from the Name column. | The supplied text does not include the control’s label. |
| MC.03.06.b023 | Make changes to users or groups, then use the undo control before closing the Info window. | Adding or removing a user or group can be undone. | The undo control’s label is missing from the supplied text. |
| MC.03.06.b024 | Click Dock to open a Finder window, then click iCloud Drive in the sidebar. | The iCloud Drive folder opens; an error message appears at its top if files or folders have permission problems. | The issue concerns files or folders in iCloud Drive that the user does not have permission to access. |
| MC.03.06.b025 | Click Repair in the error message. | The Mac repairs the files or folders, and they sync again. | An iCloud Drive permission error is displayed. The supplied text ends before completing its description of what happens after repair. |

### MC.03.07 Copy/move/conflict

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).


#### If you can’t move or copy an item on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp1770/27/mac/27) · macOS 27 text present: no

Troubleshooting steps for moving or copying files and folders, including checks for open files, storage, dragging, desktop sorting, drive formats, and permissions.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.03.07.e001 | Finder | menu | macOS menu bar | The menu used to access the View menu and desktop sort options. |
| MC.03.07.e002 | View | menu item | Finder menu | Contains the Sort By option. |
| MC.03.07.e003 | Sort By | menu item | Finder > View | Provides desktop sorting options. |
| MC.03.07.e004 | None | menu item | Finder > View > Sort By | The option to choose when desktop items return to their original positions after being moved. |
| MC.03.07.e005 | Use Stacks | setting | Desktop sorting options | The page says to make sure this isn’t selected if Sort By isn’t available. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.03.07.b001 | You can’t move or copy a file or folder. | Check that the file isn’t currently open in another app. | Basic troubleshooting step. |
| MC.03.07.b002 | You can’t move or copy a file or folder. | Verify that the destination folder or disk has enough available space. | Basic troubleshooting step. |
| MC.03.07.b003 | Dragging a file opens it instead of moving it. | Try using lighter pressure when dragging. | The page says this may happen when using Force Touch. |
| MC.03.07.b004 | Dragging items doesn’t work. | Select the item, press Command-C, go to the destination folder, press Command-V, then delete the original item. | Presented as the copy-and-delete method. |
| MC.03.07.b005 | A file or folder returns to its original desktop position after being moved. | Go to the desktop and choose Finder > View > Sort By > None. | The page says this may be due to desktop sort settings. |
| MC.03.07.b006 | You need to show the desktop. | Press F11. | Given as an alternative to going to the desktop. |
| MC.03.07.b007 | Sort By isn’t available. | Make sure Use Stacks isn’t selected. | Applies to the desktop sorting instructions. |
| MC.03.07.b008 | You can’t move an item to an external drive. | Make sure the drive is formatted as APFS, Mac OS Extended, or exFAT. | The page identifies these formats as supported for this purpose. |
| MC.03.07.b009 | You try to move files to an NTFS drive. | You can’t move files to it because NTFS drives are read-only on Mac. | Applies to NTFS drives. |
| MC.03.07.b010 | You can’t move an item to a network location. | See “Connect your Mac to shared computers and servers.” | Applies to network locations. |
| MC.03.07.b011 | The basic steps don’t work. | Check and modify permissions for the item and its destination location if needed. | The page identifies permissions as a possible cause. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.03.07.k001 | Command-C | Copy the selected item. | Copy-and-delete method. |
| MC.03.07.k002 | Command-V | Paste the copied item. | After navigating to the destination folder. |
| MC.03.07.k003 | F11 | Show the desktop. | Alternative to going to the desktop. |

#### Organize files in folders on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mh26885/27/mac/27) · macOS 27 text present: no

Apple Support instructions for creating folders, moving or copying items, grouping selected items into a folder, and merging folders with the same name.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.03.07.e006 | Dock | other | Mac screen | Clicking the Dock opens a Finder window. |
| MC.03.07.e007 | Finder | other | Finder window | Used to navigate to a location and organize files. |
| MC.03.07.e008 | File | menu | Finder menu bar | Contains the New Folder and Duplicate commands. |
| MC.03.07.e009 | New Folder | menu item | File menu | Creates a folder in the current location; the command can be dimmed when a folder cannot be created there. |
| MC.03.07.e010 | New Folder with Selection | menu item | Contextual menu for selected items | Creates a folder containing the selected items. |
| MC.03.07.e011 | Duplicate | menu item | File menu | Makes a copy of an item within the same folder. |
| MC.03.07.e012 | Merge | button | Dialog shown when moving a folder onto a folder with the same name | Merges the folders when available. |
| MC.03.07.e013 | Stop | button | Dialog shown when merging folders with different versions of identically named files | One of the stated options in that situation. |
| MC.03.07.e014 | Replace | button | Dialog shown when merging folders with different versions of identically named files | One of the stated options in that situation. |
| MC.03.07.e015 | Smart Folder | other | Referenced at the end of the article | A way to keep a handy list of files together based on specified criteria. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.03.07.b012 | Click Dock. | A Finder window opens. | On a Mac. |
| MC.03.07.b013 | Navigate in a Finder window to the desired location, or click the desktop. | The chosen location is where the new folder can be created. | The desktop can be used instead of a Finder location. |
| MC.03.07.b014 | Choose File > New Folder, or press Shift-Command-N. | A new folder is created in the current location. | If New Folder is dimmed, a folder cannot be created in the current location. |
| MC.03.07.b015 | Enter a folder name and press Return. | The folder receives the entered name. | After creating a folder. |
| MC.03.07.b016 | Drag an item to a folder. | The item is put in the folder. | In Finder. |
| MC.03.07.b017 | Select several items and drag one of them to a folder. | All selected items move to the folder. | In Finder. |
| MC.03.07.b018 | Move the pointer to the immediate left of a window title until an icon appears, then drag the icon to a folder. | The window’s content is put in the folder. | In a window; the icon appears as the pointer moves to the specified title area. |
| MC.03.07.b019 | Press and hold Shift while moving the pointer into the title area. | The window-content icon appears immediately. | In the window title area. |
| MC.03.07.b020 | Drag the beginning of a window title to a folder. | The window’s content is put in the folder. | The article says this can be done without waiting for the icon to appear. |
| MC.03.07.b021 | Select an item, press and hold Option, then drag the item to a folder. | A copy is put in the folder and the item remains in its original location. | In Finder. |
| MC.03.07.b022 | Press and hold Option and Command, then drag an item to a folder. | An alias for the item is created in the folder. | In Finder. |
| MC.03.07.b023 | Select an item and choose File > Duplicate, or press Command-D. | A copy of the item is made within the same folder. | In Finder. |
| MC.03.07.b024 | Drag files to a disk. | The files are copied to the different disk. | When the destination is a different disk. |
| MC.03.07.b025 | Press and hold Command while dragging files to a disk. | The files are moved to the different disk. | When the destination is a different disk. |
| MC.03.07.b026 | Select the items to group, then Control-click one of the selected items and choose New Folder with Selection. | A new folder is created containing the selected items. | Can be done on the desktop or in a Finder window. |
| MC.03.07.b027 | Enter a name for the newly created folder and press Return. | The folder receives the entered name. | After choosing New Folder with Selection. |
| MC.03.07.b028 | Press and hold Option while dragging one folder to the location containing a folder with the same name, then click Merge. | The folders are merged into a single folder. | The folders are at different locations. Merge appears only if one folder contains items not in the other. |
| MC.03.07.b029 | Attempt to merge folders that contain different versions of identically named files. | The only options are Stop or Replace. | The folders have the same name and contain different versions of identically named files. |
| MC.03.07.b030 | Use a Smart Folder with specified criteria. | It keeps a handy list of files together that have something in common. | The criteria are specified by the user. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.03.07.k004 | Shift-Command-N | Create a new folder. | Finder or the desktop, at the location where the folder should be created. |
| MC.03.07.k005 | Return | Confirm the entered folder name. | After creating a folder or a folder with selected items. |
| MC.03.07.k006 | Shift (press and hold) | Make the window-content icon appear immediately. | While moving the pointer into a window’s title area. |
| MC.03.07.k007 | Option (press and hold) + drag | Put a copy of an item in a folder while keeping the original in its location. | Dragging an item to a folder. |
| MC.03.07.k008 | Option-Command (press and hold) + drag | Create an alias for an item in a folder. | Dragging an item to a folder. |
| MC.03.07.k009 | Command-D | Duplicate an item. | Makes a copy within the same folder. |
| MC.03.07.k010 | Command (press and hold) + drag | Move files to a different disk. | Dragging files to the destination disk. |
| MC.03.07.k011 | Control-click | Open the contextual menu for a selected item. | Click one of the selected items before choosing New Folder with Selection. |
| MC.03.07.k012 | Option (press and hold) + drag | Move a folder to the location containing a same-named folder to bring up the merge dialog. | The folders are at different locations. |

### MC.03.08 Trash & recovery

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).


#### Delete files and folders on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp1093/27/mac/27) · macOS 27 text present: no

Explains how to move items to the Trash, permanently delete them, restore them before emptying the Trash, suppress the emptying warning, and automatically remove Trash items after 30 days.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.03.08.e001 | Trash | icon | Dock | Destination for items moved from the Mac; items remain there until the Trash is emptied. |
| MC.03.08.e002 | Continue | button | Confirmation prompt for moving a locked item to the Trash | Confirms that you want to move the locked item to the Trash. |
| MC.03.08.e003 | File > Get Info | menu item | File menu | Opens the selected item's information so it can be unlocked. |
| MC.03.08.e004 | Locked | control | Item information | Checkbox to deselect when unlocking an item. |
| MC.03.08.e005 | Touch ID | control | Authorization prompt | May be required if you are not logged in as an administrator. |
| MC.03.08.e006 | Apple Watch | control | Authorization prompt | May be used for authorization if you are not logged in as an administrator. |
| MC.03.08.e007 | Trash | icon | Dock | Click to open the Trash window. |
| MC.03.08.e008 | Delete Immediately | menu item | Context menu for an item in the Trash | Deletes the selected Trash item immediately. |
| MC.03.08.e009 | Delete | button | Warning message after choosing Delete Immediately | Confirms deletion of the item. |
| MC.03.08.e010 | Empty | button | Upper-right corner of the Finder window showing the Trash | Starts emptying all items from the Trash. |
| MC.03.08.e011 | Empty Trash | menu item | Finder | An alternative action for emptying all items from the Trash. |
| MC.03.08.e012 | Empty Trash | button | Warning message after starting to empty all Trash items | Confirms emptying the Trash. |
| MC.03.08.e013 | Finder settings | setting | Finder | Includes an option to automatically empty the Trash. |
| MC.03.08.e014 | Finder > Settings | menu item | Finder menu | Opens Finder settings. |
| MC.03.08.e015 | Advanced | panel | Finder settings | Contains the warning setting mentioned in the article. |
| MC.03.08.e016 | Show warning before emptying the Trash. | setting | Finder settings > Advanced | Deselect to prevent the Trash warning from appearing. |
| MC.03.08.e017 | File > Put Back | menu item | File menu | Returns a selected item from the Trash. |
| MC.03.08.e018 | Remove items from the Trash after 30 days. | setting | Finder settings | Automatically removes items from the Trash after 30 days. |
| MC.03.08.e019 | iCloud Drive | other | Mac storage | Items moved to the Trash from iCloud Drive are automatically removed from the Trash after 30 days. |
| MC.03.08.e020 | Time Machine | other | Backup reference | Mentioned as a way to create backups before deleting items. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.03.08.b001 | Drag an item to the Trash in the Dock, or select it and press Command-Delete. | The item is moved to the Trash; it is not deleted until the Trash is emptied. | Applies to items on the Mac or in iCloud Drive. |
| MC.03.08.b002 | Move a locked item to the Trash. | Confirm by clicking Continue, or unlock the item before putting it in the Trash. | Applies when the item is locked. |
| MC.03.08.b003 | Select a locked item, choose File > Get Info, and deselect the Locked checkbox. | The item is unlocked so it can be put in the Trash. | Applies to a locked item. |
| MC.03.08.b004 | Attempt an action that requires authorization. | You may need to click Touch ID or your Apple Watch. | If you are not logged in as an administrator. |
| MC.03.08.b005 | Control-click an item in the Trash and choose Delete Immediately. | A warning message appears; click Delete to delete that item. | Applies to emptying one item from the Trash. |
| MC.03.08.b006 | Click the Empty button in the upper-right corner of the Finder window showing the Trash, or choose Empty Trash. | A warning message appears; click Empty Trash to confirm emptying the Trash. | Applies to emptying all Trash items. |
| MC.03.08.b007 | Move items from the Mac to the Trash. | The items remain in the Trash until it is emptied. | Items moved from iCloud Drive are subject to the separate 30-day removal behavior. |
| MC.03.08.b008 | Empty the Trash. | Items emptied from the Trash are permanently deleted and are no longer available. | — |
| MC.03.08.b009 | Move items from iCloud Drive to the Trash. | They are automatically emptied from the Trash after 30 days, or can be emptied sooner. | This applies regardless of Finder settings. |
| MC.03.08.b010 | Press Option while clicking Empty or choosing Empty Trash. | The Trash warning message does not appear for that action. | Once only. |
| MC.03.08.b011 | In Finder > Settings, click Advanced and deselect “Show warning before emptying the Trash.” | The Trash warning message is prevented from appearing. | Always, until the setting is changed. |
| MC.03.08.b012 | Before emptying the Trash, drag an item out of the Trash or select it and choose File > Put Back. | The item is restored from the Trash. | Applies only before the Trash is emptied. |
| MC.03.08.b013 | In Finder, select “Remove items from the Trash after 30 days.” | Items are automatically removed from the Trash after 30 days. | Items automatically removed are permanently deleted and are no longer available. |
| MC.03.08.b014 | Turn off Time Machine or go without a recent backup before deleting items. | The article advises storing a backup copy on a storage device in case you later want the item. | Advice presented as a note. |
| MC.03.08.b015 | Want to restore an item deleted from iCloud Drive. | The article refers to “Use iCloud.com to restore and recover iCloud information” in the iCloud User Guide. | — |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.03.08.k001 | Command-Delete | Move the selected item to the Trash. | On a Mac. |
| MC.03.08.k002 | Command-I | Open Get Info for the selected item. | When unlocking an item before putting it in the Trash. |
| MC.03.08.k003 | Option | Suppress the Trash warning for that action. | Hold while clicking Empty or choosing Empty Trash. |

### MC.03.09 Desktop & stacks

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).

Also covered by: [Use desktop stacks on Mac](https://support.apple.com/guide/mac-help/mh35846/27/mac/27)


#### Ways to organize files on your Mac desktop

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mh35951/27/mac/27) · macOS 27 text present: no

The page describes grouping desktop items into folders, using stacks, arranging files, and changing desktop view options.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.03.09.e001 | desktop | view | Mac desktop | The area where files, folders, and aliases can be kept and organized. |
| MC.03.09.e002 | New Folder with Selection | menu item | Context menu after Control-clicking a selected desktop item | Creates a folder containing the selected items. |
| MC.03.09.e003 | View | menu | Finder menu bar after clicking the desktop | Contains the desktop arrangement and view-options commands described on this page. |
| MC.03.09.e004 | Sort By | menu item | View menu | Opens choices for sorting desktop files. |
| MC.03.09.e005 | Name | menu item | View > Sort By | Sorts desktop files by name. |
| MC.03.09.e006 | Kind | menu item | View > Sort By | Sorts desktop files by kind. |
| MC.03.09.e007 | Date Added | menu item | View > Sort By | Sorts desktop files by date added. |
| MC.03.09.e008 | Size | menu item | View > Sort By | Sorts desktop files by size. |
| MC.03.09.e009 | Tags | menu item | View > Sort By | Sorts desktop files by tags. |
| MC.03.09.e010 | None | menu item | View > Sort By | Leaves file placement under the user's control. |
| MC.03.09.e011 | Clean Up By | menu item | View menu | Opens choices for arranging desktop files. |
| MC.03.09.e012 | Show View Options | menu item | View menu or desktop context menu | Opens controls for changing desktop display options. |
| MC.03.09.e013 | iCloud Desktop and Documents | setting | Mentioned in the page's tip; no settings path specified | When turned on, desktop items can be available on another Mac using the same iCloud account when the setting is also turned on there. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.03.09.b001 | Select the desktop items to group, then Control-click one of them and choose New Folder with Selection. | The selected items are grouped into a new folder. | Locked items can't be grouped. |
| MC.03.09.b002 | Group desktop stacks by kind. | Items such as images and presentations are grouped into separate stacks by kind. | Stacks must be used and grouped by kind. |
| MC.03.09.b003 | Add a new file to the desktop while stacks are in use. | The file goes immediately into the correct stack. | Stacks must be used. |
| MC.03.09.b004 | Group stacks by date or tags. | Stacks group desktop files by date or tags. | Stacks must be used. |
| MC.03.09.b005 | Click the desktop, choose View > Sort By, then choose an option. | Desktop files are sorted by the chosen criterion, such as name, kind, date added, size, or tags. | — |
| MC.03.09.b006 | Set View > Sort By to None. | You can control the placement of files on the desktop. | — |
| MC.03.09.b007 | Click the desktop, choose View > Clean Up By, then choose how to arrange the files. | Files are arranged according to the chosen option. | — |
| MC.03.09.b008 | Click the desktop, choose View > Show View Options, then use the controls. | You can change icon size, grid spacing, text size, and other options. | — |
| MC.03.09.b009 | Control-click the desktop, then choose Show View Options. | The view-options controls are available for changing desktop display options. | — |
| MC.03.09.b010 | Turn on iCloud Desktop and Documents on a Mac, then sign in with the same iCloud account and turn on iCloud Desktop and Documents on another Mac. | You can get the same desktop items on the other Mac. | Both Macs use the same iCloud account and have iCloud Desktop and Documents turned on. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.03.09.k001 | Control-click | Open the context menu for a desktop item or the desktop. | Use on a selected item to choose New Folder with Selection, or on the desktop to choose Show View Options. |

## 14.4 MC.04 Window model


### MC.04.01 App vs window relationship

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).

Also covered by: [Move and arrange app windows on Mac](https://support.apple.com/guide/mac-help/mchlp2469/27/mac/27), [Use tabs in windows on Mac](https://support.apple.com/guide/mac-help/mchla4695cce/27/mac/27)


#### Quit apps on Mac - Apple Support

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchl834d18c2/27/mac/27) · macOS 27 text present: no

The page explains how to quit an app from the menu bar or with Command-Q, and notes that open windows may reappear the next time the app is opened. The provided text ends before completing a second instruction.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.04.01.e001 | menu bar | view | Top of the screen | The menu bar is where the app’s Quit command can be chosen. |
| MC.04.01.e002 | Preview | menu | Menu bar | Example app menu used in the instruction to quit Preview. |
| MC.04.01.e003 | Quit Preview | menu item | Preview menu in the menu bar | Example command for quitting Preview. |
| MC.04.01.e004 | Apple menu | menu | Not specified | Named as the start of an incomplete instruction about closing app windows when quitting. |
| MC.04.01.e005 | See also | view | Below the article text | Section listing related support topics. |
| MC.04.01.e006 | Open apps on Mac | other | See also section | Related support topic. |
| MC.04.01.e007 | If an app freezes or quits unexpectedly on Mac | other | See also section | Related support topic. |
| MC.04.01.e008 | Move and arrange app windows on Mac | other | See also section | Related support topic. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.04.01.b001 | Choose App Name > Quit App in the menu bar. | The app quits. | The command uses the app’s name in place of “App Name” and “App.” |
| MC.04.01.b002 | Press Command-Q. | The app quits. | The shortcut is given as an alternative to choosing the Quit command. |
| MC.04.01.b003 | Quit an app while it has windows open, then open the app again. | The app’s windows are shown again. | The app had windows open when it was quit. |
| MC.04.01.b004 | Choose the Apple menu to have app windows close and not be shown the next time apps are opened. | The text indicates that app windows can be made to close and not reappear the next time apps are opened. | The provided instruction is truncated after “choose Apple menu”; the remaining command or setting is not shown. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.04.01.k001 | Command-Q | Quit the app. | Alternative to choosing App Name > Quit App in the menu bar. |

#### Mac keyboard shortcuts - Apple Support

T1 · OS documentation · [Apple Support](https://support.apple.com/en-us/102650) · macOS 27 text present: yes

Apple Support reference for using Mac keyboard shortcuts, including common commands, Finder and system shortcuts, text editing, accessibility, and power controls. Published September 14, 2026.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.04.01.e009 | Using keyboard shortcuts | section | Article | Explains modifier keys and how to press a shortcut. |
| MC.04.01.e010 | Cut, copy, paste, and other common shortcuts | section | Article | Lists common app and system shortcuts. |
| MC.04.01.e011 | Sleep, log out, and shut down shortcuts | section | Article | Lists power, lock, logout, and shutdown controls. |
| MC.04.01.e012 | Finder and system shortcuts | section | Article | Lists Finder, window, display, and system shortcuts. |
| MC.04.01.e013 | Window-tiling shortcuts | section | Finder and system shortcuts | A referenced shortcut topic; no individual window-tiling shortcuts are provided in the supplied text. |
| MC.04.01.e014 | Text-editing shortcuts | section | Article | Lists text formatting, cursor movement, selection, and editing shortcuts. |
| MC.04.01.e015 | Accessibility shortcuts | section | Article | Contains vision, keyboard-focus, and other accessibility shortcut topics. |
| MC.04.01.e016 | Vision shortcuts | section | Accessibility shortcuts | Lists shortcuts for inverting colors and changing contrast. |
| MC.04.01.e017 | Keyboard focus shortcuts | section | Accessibility shortcuts | Lists shortcuts for moving keyboard focus. |
| MC.04.01.e018 | Other accessibility shortcuts | section | Accessibility shortcuts | References the Accessibility Shortcuts panel and other accessibility topics. |
| MC.04.01.e019 | Fn (Function) or Globe | other | Modifier-key list | A key that can be used in shortcuts. |
| MC.04.01.e020 | Control (or Ctrl) ⌃ | other | Modifier-key list | A modifier key. |
| MC.04.01.e021 | Option (or Alt) ⌥ | other | Modifier-key list | A modifier key. |
| MC.04.01.e022 | Shift ⇧ | other | Modifier-key list | A modifier key. |
| MC.04.01.e023 | Caps Lock ⇪ | other | Modifier-key list | A listed key. |
| MC.04.01.e024 | Command (or Cmd) ⌘ | other | Modifier-key list | A modifier key. |
| MC.04.01.e025 | Esc (Escape) ⎋ | other | Modifier-key list | A listed key. |
| MC.04.01.e026 | Tab ⇥ | other | Modifier-key list | A listed key. |
| MC.04.01.e027 | Delete ⌫ | other | Modifier-key list | A listed key. |
| MC.04.01.e028 | Return ⏎ | other | Modifier-key list | A listed key. |
| MC.04.01.e029 | Up Arrow ▲ | other | Modifier-key list | A listed key. |
| MC.04.01.e030 | Down Arrow ▼ | other | Modifier-key list | A listed key. |
| MC.04.01.e031 | Left Arrow ◀ | other | Modifier-key list | A listed key. |
| MC.04.01.e032 | Right Arrow ▶ | other | Modifier-key list | A listed key. |
| MC.04.01.e033 | Spotlight search field | control | Spotlight | Shown or hidden with Command–Space bar; Siri AI (Beta) can also start a conversation from it when turned on. |
| MC.04.01.e034 | Character Viewer | window | System | Provides emoji and other symbols. |
| MC.04.01.e035 | Quick Look | view | Finder | Previews selected items. |
| MC.04.01.e036 | Dock | view | Desktop | Can be shown or hidden and can use magnification. |
| MC.04.01.e037 | Control Center | view | System | Shown or hidden with Fn-C. |
| MC.04.01.e038 | Notification Center | view | System | Shown or hidden with Fn-N. |
| MC.04.01.e039 | Apps | view | System | Shown or hidden with Fn-Shift-A in macOS Tahoe 26 or later. |
| MC.04.01.e040 | Launchpad | view | System | Shown or hidden with Fn-Shift-A in earlier macOS versions. |
| MC.04.01.e041 | Type to Siri (Siri Classic) | view | System | Opened or closed by pressing Command twice. |
| MC.04.01.e042 | Accessibility Shortcuts panel | panel | Accessibility | Shown with Option-Command-F5 or a triple-press of Touch ID. |
| MC.04.01.e043 | Apple menu  | menu | Menu bar | Referenced for System Settings, logout, shutdown, and restart actions. |
| MC.04.01.e044 | Go menu | menu | Finder menu bar | Provides shortcuts for commonly used folders. |
| MC.04.01.e045 | System Settings | window | Apple menu  | The current name used in the article for system preferences. |
| MC.04.01.e046 | System Preferences | window | Apple menu  | The alternate, earlier name given in the article. |
| MC.04.01.e047 | Keyboard | panel | System Settings or System Preferences | Contains Keyboard Shortcuts and keyboard-related settings. |
| MC.04.01.e048 | Keyboard Shortcuts | panel | Keyboard settings | The panel used to enable or select certain shortcuts. |
| MC.04.01.e049 | Accessibility | panel | Keyboard Shortcuts | Contains the Invert colors and Contrast shortcut options. |
| MC.04.01.e050 | Invert colors | setting | Keyboard Shortcuts > Accessibility | An option to select for the vision shortcut. |
| MC.04.01.e051 | Contrast | setting | Keyboard Shortcuts > Accessibility | An option to select for the vision shortcuts. |
| MC.04.01.e052 | Displays settings | panel | System Settings | Opened by Option–Brightness Up. |
| MC.04.01.e053 | Mission Control settings | panel | System Settings | Opened by Option–Mission Control. |
| MC.04.01.e054 | Sound settings | panel | System Settings | Opened by Option–Volume Up or another volume key with Option. |
| MC.04.01.e055 | Keyboard settings | panel | System Settings | Opened by Option–Keyboard Brightness Up. |
| MC.04.01.e056 | Accessibility Shortcuts | other | Other accessibility shortcuts | Named topic in the article. |
| MC.04.01.e057 | VoiceOver | other | Other accessibility shortcuts | The article references VoiceOver commands. |
| MC.04.01.e058 | Mouse Keys | other | Other accessibility shortcuts | The article references using Mouse Keys to control the pointer. |
| MC.04.01.e059 | Touch ID | other | Keyboard and power controls | On keyboards with Touch ID, pressing it locks the screen; triple-pressing it opens the Accessibility Shortcuts panel. |
| MC.04.01.e060 | Power button | button | Mac or built-in keyboard | Its behavior depends on the Mac and keyboard configuration. |
| MC.04.01.e061 | Brightness Up | button | Keyboard top row | Used in display brightness shortcuts and to open Displays settings with Option. |
| MC.04.01.e062 | Brightness Down | button | Keyboard top row | Used in display brightness and video-mirroring shortcuts. |
| MC.04.01.e063 | Mission Control | button | Keyboard top row | Used in Mission Control and desktop shortcuts. |
| MC.04.01.e064 | Volume Up | button | Keyboard top row | Used in sound settings and volume adjustment shortcuts. |
| MC.04.01.e065 | Volume Down | button | Keyboard top row | Used in volume adjustment shortcuts. |
| MC.04.01.e066 | Keyboard Brightness Up | button | Keyboard top row | Used to open Keyboard settings or adjust keyboard brightness. |
| MC.04.01.e067 | Keyboard Brightness Down | button | Keyboard top row | Used to adjust keyboard brightness. |
| MC.04.01.e068 | Sidebar | view | Finder windows | Shown or hidden with Option-Command-S. |
| MC.04.01.e069 | Preview pane | view | Finder windows | Shown or hidden with Shift-Command-P. |
| MC.04.01.e070 | Path bar | view | Finder windows | Shown or hidden with Option-Command-P. |
| MC.04.01.e071 | Status bar | view | Finder windows | Shown or hidden with Command-Slash (/). |
| MC.04.01.e072 | Toolbar | view | App or Finder window | Can be shown or hidden using the listed toolbar shortcuts. |
| MC.04.01.e073 | tab bar | view | Finder window | Shown or hidden with the relevant Command-T or Shift-Command-T shortcut, depending on context. |
| MC.04.01.e074 | Get Info window | window | Finder | Shown for a selected file with Command-I. |
| MC.04.01.e075 | Go to Folder window | window | Finder | Opened with Shift-Command-G. |
| MC.04.01.e076 | Connect to Server window | window | Finder | Opened with Command-K. |
| MC.04.01.e077 | Computer window | window | Finder | Opened with Shift-Command-C. |
| MC.04.01.e078 | Recents window | window | Finder | Opened with Shift-Command-F. |
| MC.04.01.e079 | Fonts window | window | Text editing | Shown or hidden with Command-T. |
| MC.04.01.e080 | Spelling and Grammar window | window | Text editing | Shown with Shift-Command-Colon (:). |
| MC.04.01.e081 | Find window | window | App | Opened with Command-F where applicable. |
| MC.04.01.e082 | Print dialog | sheet | App | Opened with Command-P. |
| MC.04.01.e083 | Save As dialog | sheet | App | Shown with Shift-Command-S in applicable apps. |
| MC.04.01.e084 | Open dialog | sheet | App | Command-O can open a dialog to select a file; Command-D selects the Desktop folder within Open or Save dialogs. |
| MC.04.01.e085 | Save dialog | sheet | App | Command-D selects the Desktop folder within it. |
| MC.04.01.e086 | Dock magnification | setting | Dock | Can be turned on or off while Control-Shift is held when accessing the Dock. |
| MC.04.01.e087 | Siri AI (Beta) | setting | Siri and Spotlight | When turned on, enables the mentioned Siri actions from Spotlight and the selected area or active window shortcuts. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.04.01.b005 | Press and hold the modifier keys in a shortcut, then press the last key; release the keys. | Performs the shortcut action. | The action may vary by app; shortcuts in one app might not work in another. |
| MC.04.01.b006 | Use a Windows PC keyboard for a Mac shortcut. | Use Alt instead of Option, and Ctrl or the Windows logo key instead of Command. | Applies to keyboards made for Windows PCs. |
| MC.04.01.b007 | Press a top-row key with Fn (Function) or Globe. | Uses the key as a standard function key such as F1, F2, or F3. | For top-row keys with special symbols and functions. |
| MC.04.01.b008 | Press Option with certain top-row keys. | Quickly opens certain system settings. | Only certain top-row keys support this. |
| MC.04.01.b009 | Use a shortcut shown for an app or action. | The described operation occurs. | Behavior may vary with the app; some shortcuts require the app to support the feature. |
| MC.04.01.b010 | Press the Power button. | Turns on the Mac or wakes it from sleep. | General power-button behavior. |
| MC.04.01.b011 | Press and hold the Power button. | Forces the Mac to turn off. | For example, when the Mac is not responding and Shut Down cannot be chosen from the Apple menu. |
| MC.04.01.b012 | Press Touch ID. | Locks the screen. | On keyboards with Touch ID. |
| MC.04.01.b013 | Press and hold the Power button for 1.5 seconds. | Puts the Mac to sleep. | On built-in keyboards without Touch ID. |
| MC.04.01.b014 | Hold Option while choosing Log Out, Shut Down, or Restart from the Apple menu . | Completes the action without showing a confirmation dialog. | When choosing one of the named Apple menu actions. |
| MC.04.01.b015 | Hold Option while double-clicking an item. | Opens the item in a separate window, then closes the original window. | Finder. |
| MC.04.01.b016 | Hold Command while double-clicking a folder. | Opens the folder in a separate tab or window. | Finder. |
| MC.04.01.b017 | Hold Command while dragging an item to another volume. | Moves the item to the other volume instead of copying it. | Finder; while dragging between volumes. |
| MC.04.01.b018 | Hold Option while dragging an item. | Copies the dragged item; the pointer changes while dragging. | Finder. |
| MC.04.01.b019 | Hold Option-Command while dragging an item. | Makes an alias of the dragged item; the pointer changes while dragging. | Finder. |
| MC.04.01.b020 | Option-click a disclosure triangle. | Opens all folders within the selected folder. | Finder list view only. |
| MC.04.01.b021 | Command-click a window title. | Shows the folders that contain the current folder. | Finder. |
| MC.04.01.b022 | Hold Command or Shift while selecting an item. | Selects multiple items. | Finder. |
| MC.04.01.b023 | Click the Go menu in the Finder menu bar. | Shows shortcuts for opening commonly used folders, including Applications, Documents, Downloads, Utilities, and iCloud Drive. | Finder. |
| MC.04.01.b024 | Press the listed Mac startup key combinations link. | The article points to information about Mac startup key combinations. | No individual startup combination is specified in the supplied text. |
| MC.04.01.b025 | Use an accessibility shortcut while VoiceOver is active. | Some shortcuts may require VoiceOver to ignore the next key press. | When using VoiceOver. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.04.01.k002 | Command-X | Cut the selected item and copy it to the Clipboard. | Common shortcut. |
| MC.04.01.k003 | Command-C | Copy the selected item to the Clipboard. | Also works for files in the Finder. |
| MC.04.01.k004 | Command-V | Paste the contents of the Clipboard into the current document or app. | Also works for files in the Finder. |
| MC.04.01.k005 | Command-Z | Undo the previous command. | Common shortcut. |
| MC.04.01.k006 | Shift-Command-Z | Redo, reversing the undo command. | Some apps support undoing and redoing multiple commands. |
| MC.04.01.k007 | Command-A | Select all items. | Common shortcut. |
| MC.04.01.k008 | Command-F | Find items in a document or open a Find window. | Common shortcut. |
| MC.04.01.k009 | Command-G | Find the next occurrence of the item previously found. | Find Again. |
| MC.04.01.k010 | Shift-Command-G | Find the previous occurrence. | Find Again. |
| MC.04.01.k011 | Command-H | Hide the windows of the front app. | Common shortcut. |
| MC.04.01.k012 | Option-Command-H | Show the front app and hide all other apps. | Common shortcut. |
| MC.04.01.k013 | Command-M | Minimize the front window to the Dock. | Common shortcut. |
| MC.04.01.k014 | Option-Command-M | Minimize all windows of the front app. | Common shortcut. |
| MC.04.01.k015 | Command-O | Open the selected item, or open a dialog to select a file to open. | Common shortcut. |
| MC.04.01.k016 | Command-P | Open a print dialog. | Print the current document. |
| MC.04.01.k017 | Command-Q | Quit the current app. | Common shortcut. |
| MC.04.01.k018 | Command-S | Save the current document. | Common shortcut. |
| MC.04.01.k019 | Command-T | Open a new tab. | Common shortcut. |
| MC.04.01.k020 | Command-W | Close the front window. | Common shortcut. |
| MC.04.01.k021 | Option-Command-W | Close all windows of the app. | Common shortcut. |
| MC.04.01.k022 | Option-Command-Esc | Force quit an app. | Common shortcut. |
| MC.04.01.k023 | Command-Space bar | Show or hide the Spotlight search field; if Siri AI (Beta) is turned on, start a conversation with Siri from the Spotlight search field. | Some versions of macOS may assign this shortcut to changing input sources when multiple input sources are used. |
| MC.04.01.k024 | Option-Command-Space bar | Perform a Spotlight search from a Finder window. | Finder. |
| MC.04.01.k025 | Control-Command-Space bar | Show the Character Viewer. | Choose emoji and other symbols. |
| MC.04.01.k026 | Fn-E | Show the Character Viewer. | Choose emoji and other symbols. |
| MC.04.01.k027 | Control-Command-F | Use or stop using the app in full screen. | If supported by the app. |
| MC.04.01.k028 | Space bar | Use Quick Look to preview the selected item. | Selected item. |
| MC.04.01.k029 | Command-Tab | Switch to the next most recently used app among open apps. | App switching. |
| MC.04.01.k030 | Command-Grave accent (`) | Switch between windows of the app being used. | The character on the second key varies by keyboard; generally the key above Tab and left of 1. |
| MC.04.01.k031 | Shift-Command-5 | Take a screenshot or make a screen recording. | macOS Mojave or later. |
| MC.04.01.k032 | Shift-Command-3 | Take a screenshot. | Common screenshot shortcut. |
| MC.04.01.k033 | Shift-Command-4 | Take a screenshot. | Common screenshot shortcut. |
| MC.04.01.k034 | Shift-Command-N | Create a new empty folder. | Finder. |
| MC.04.01.k035 | Control-Command-N | Create a new folder that contains the currently selected items. | Finder. |
| MC.04.01.k036 | Command-Comma (,) | Open settings or preferences for the front app. | App. |
| MC.04.01.k037 | Fn-Q | Create a Quick Note. | System. |
| MC.04.01.k038 | Power button | Turn on the Mac or wake it from sleep. | Power control. |
| MC.04.01.k039 | Press and hold Power button | Force the Mac to turn off. | For example, if the Mac is not responding and Shut Down cannot be chosen from the Apple menu. |
| MC.04.01.k040 | Power button | Lock the screen. | On keyboards with Touch ID. |
| MC.04.01.k041 | Press and hold Power button for 1.5 seconds | Put the Mac to sleep. | Built-in keyboards without Touch ID. |
| MC.04.01.k042 | Option-Command-Power button | Put the Mac to sleep. | Built-in keyboards without Touch ID. |
| MC.04.01.k043 | Control-Shift-Power button | Put the displays to sleep. | Built-in keyboards without Touch ID. |
| MC.04.01.k044 | Control-Power button | Show a dialog asking whether to restart, sleep, or shut down. | Built-in keyboards without Touch ID. |
| MC.04.01.k045 | Control-Command-Power button | Force the Mac to restart without prompting to save open, unsaved documents. | Built-in keyboards without Touch ID. |
| MC.04.01.k046 | Control-Option-Command-Power button | Quit all apps, then shut down the Mac; asks whether to save open documents with unsaved changes. | The article states that this is equivalent to choosing restart from the Apple menu . |
| MC.04.01.k047 | Control-Command-Q | Lock the screen. | System. |
| MC.04.01.k048 | Shift-Command-Q | Log out of the macOS user account and ask for confirmation. | Logout. |
| MC.04.01.k049 | Option-Shift-Command-Q | Log out immediately without confirming. | Logout. |
| MC.04.01.k050 | Option while choosing Log Out, Shut Down, or Restart | Complete the action without showing a confirmation dialog. | Choose the action from the Apple menu . |
| MC.04.01.k051 | Command-D | Duplicate the selected files. | Finder. |
| MC.04.01.k052 | Command-E | Eject the selected disk or volume. | Finder. |
| MC.04.01.k053 | Command-F | Start a Spotlight search in the Finder window. | Finder. |
| MC.04.01.k054 | Command-I | Show the Get Info window for a selected file. | Finder. |
| MC.04.01.k055 | Command-R | Show the original file for the selected alias. | Finder; when an alias is selected. |
| MC.04.01.k056 | Command-R | Refresh or reload the page. | Some apps, such as Calendar or Safari. |
| MC.04.01.k057 | Command-R | Check for software updates again. | Software Update. |
| MC.04.01.k058 | Shift-Command-C | Open the Computer window. | Finder. |
| MC.04.01.k059 | Shift-Command-D | Open the desktop folder. | Finder. |
| MC.04.01.k060 | Shift-Command-F | Open the Recents window, showing files viewed or changed recently. | Finder. |
| MC.04.01.k061 | Shift-Command-G | Open a Go to Folder window. | Finder. |

(171 more shortcut mentions on this page are in `macos_shortcuts.csv`.)

macOS 27 statements on this page:

- MC.04.01.n01: The page states that in macOS 27 Golden Gate or later, with Siri AI (Beta) turned on, Shift-Command-6 asks Siri about the selected area.
- MC.04.01.n02: The page states that in macOS 27 Golden Gate or later, with Siri AI (Beta) turned on, Shift-Command-Space bar asks Siri about the active window.

System Settings paths referenced: Apple menu  > System Settings (or System Preferences) > Keyboard > Keyboard Shortcuts > Accessibility > “Invert colors” and “Contrast”; Apple menu  > System Settings (or System Preferences) > Keyboard > Keyboard Shortcuts > Keyboard; Displays settings; Mission Control settings; Sound settings; Keyboard settings

#### Windows

T1 · HIG (developer guidance) · [Apple HIG](https://developer.apple.com/design/human-interface-guidelines/windows) · macOS 27 text present: no

The page describes window concepts and best practices across Apple platforms, with macOS guidance on window anatomy, states, appearance, and bottom bars. It does not mention macOS 27.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.04.01.e088 | window | window | Throughout | Presents an app’s views and components. In macOS, a window has a frame and a body area. |
| MC.04.01.e089 | frame | other | Above the body area of a macOS window | Can include window controls and a toolbar; in rare cases, it can also include a bottom bar. |
| MC.04.01.e090 | body area | other | Within a macOS window, below the frame | The area containing window content. |
| MC.04.01.e091 | window controls | control | In the macOS window frame | System-provided controls for closing, minimizing, and zooming a window. |
| MC.04.01.e092 | toolbar | other | May appear in the macOS window frame | A window component that can appear alongside window controls. |
| MC.04.01.e093 | bottom bar | other | Below body content in some macOS windows | A rare part of the window frame. The page advises using it only for a small amount of information related to the window’s contents or a selected item. |
| MC.04.01.e094 | Main | other | macOS window state | The app’s frontmost window; an app can have only one main window. |
| MC.04.01.e095 | Key | other | macOS window state | The active window that accepts input; only one key window can be onscreen at a time. |
| MC.04.01.e096 | Inactive | other | macOS window state | A window that is not in the foreground. |
| MC.04.01.e097 | title bar options | control | In the macOS window title bar | Options for closing, minimizing, and zooming. The key window uses color for these options; inactive windows and main windows that are not key use gray. |
| MC.04.01.e098 | panel | window | Example: a panel floating above a macOS main window | A window that can become key instead of the app’s main window. Panels such as Colors or Fonts may become key when their title bar or a component requiring keyboard input is clicked. |
| MC.04.01.e099 | vibrancy | other | macOS window appearance | An effect that can pull color into a window from the content underneath it. Inactive windows do not use vibrancy. |
| MC.04.01.e100 | inspector | view | Typically on the trailing side of a split view | Suggested as an alternative place for information that does not fit in a small bottom bar. |
| MC.04.01.e101 | status bar | other | Example of a Finder bottom bar | Displays the total number of items in a window, the number selected, and available disk space. |

| ID | Context | Guideline | Conditions |
|---|---|---|---|
| MC.04.01.b026 | When designing windows for different sizes and multitasking workflows | Make windows adapt fluidly to different sizes. | General guidance that includes macOS. |
| MC.04.01.b027 | When deciding whether to open a new window | Choose the right moment; avoid opening windows excessively or making new windows the default unless that benefits the app experience. | General guidance that includes macOS. |
| MC.04.01.b028 | When content could be useful in a separate window | Consider letting people open it in a new window using a context-menu command or a File-menu command. | General guidance that includes macOS; the page references Open for developer guidance. |
| MC.04.01.b029 | When implementing window frames or controls | Avoid custom window UI; use system-provided windows and controls rather than replicating their appearance. | General guidance that includes macOS. |
| MC.04.01.b030 | When referring to app windows in user-facing content | Use the term “window”; avoid alternative terms such as “scene.” | General guidance that includes macOS. |
| MC.04.01.b031 | When creating custom macOS windows | Use system-defined appearances. If using custom implementations, update the background and button appearances when the window changes state. | System-provided components update these appearances automatically. |
| MC.04.01.b032 | When considering what information or actions to place in a macOS bottom bar | Do not put critical information or actions there, because relocating a window can hide its bottom edge. If necessary, use the bar only for a small amount of directly related information. | A bottom bar is optional and rare. |
| MC.04.01.b033 | When more information is needed than fits in a macOS bottom bar | Consider using an inspector, typically on the trailing side of a split view. | MacOS guidance. |

API names: `Open`, `collectionsceneActivationConfigurationForItemAt:point:)`, `UIWindow.ActivationInteraction`, `QLPreviewActivationConfiguration`, `PlainWindow`, `Default`, `Positioning and sizing windows`, `Volumetric`, `ornament(visibility:attachmentAnchor:contentAlignment:ornament:)`, `WindowGroup`, `UIWindow`, `NSWindow`

#### Independent reports (MC.04.01)

| ID | Kind | Area | Claim | Source |
|---|---|---|---|---|
| MC.04.01.x01 | new | other | Golden Gate includes a dynamic Golden Gate wallpaper with sweeping curves and a Golden Gate Sunset wallpaper. The latter has an animated screen saver. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/3/) |
| MC.04.01.x02 | behavior | other | The Mac’s stylized wallpaper also appears on other Apple devices, giving the platforms a shared look. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/3/) |
| MC.04.01.x03 | behavior | other | When viewing a schedule, an “add to calendar” button may appear; the writer also saw it after suggesting a date and time to meet friends. | T4 [Tom's Guide](https://www.tomsguide.com/computing/macos/macos-golden-gate-review) |
| MC.04.01.x04 | new | other | In Photos editing mode, Reframe is available from a new “Tools” option. | T4 [Tom's Guide](https://www.tomsguide.com/computing/macos/macos-golden-gate-review) |
| MC.04.01.x05 | behavior | other | In Reframe, side sliders provide more precise control over a photo’s angle, orientation, or size. | T4 [Tom's Guide](https://www.tomsguide.com/computing/macos/macos-golden-gate-review) |

### MC.04.02 Active/inactive appearance

Status: **PARTIAL**. Corner radius and shadow changes reported qualitatively; no numeric values published (GAP-B5-07).

Also covered by: [Change Desktop & Dock settings on Mac - Apple Support](https://support.apple.com/guide/mac-help/mchlp1119/27/mac/27), [Windows](https://developer.apple.com/design/human-interface-guidelines/windows)


#### Change Appearance settings on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp1225/27/mac/27) · macOS 27 text present: no

Use Appearance settings to change the overall look of macOS and choose how scroll bars behave.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.04.02.e001 | Apple menu | menu | At the start of the instructions | The instructions say to choose the Apple menu; the menu path following it is not shown. |
| MC.04.02.e002 | Open Appearance settings for me | button | Below the introductory instructions | Opens Appearance settings. |
| MC.04.02.e003 | Appearance | setting | Appearance settings | Choose the appearance for buttons, menus, and windows. |
| MC.04.02.e004 | Light | setting | Appearance setting | Provides a light appearance that doesn’t change. |
| MC.04.02.e005 | Dark | setting | Appearance setting | Provides a darkened appearance so content stands out while windows and controls seem to recede. |
| MC.04.02.e006 | Auto | setting | Appearance setting | Switches the appearance based on the Night Shift schedule, or sunrise and sunset if no schedule is set. |
| MC.04.02.e007 | Liquid Glass | setting | Appearance settings | A slider for adjusting the appearance of app icons. |
| MC.04.02.e008 | Color | setting | Appearance settings | Choose a color for buttons, pop-up menus, and other UI controls. |
| MC.04.02.e009 | Text highlight color | setting | Appearance settings | Choose a color for highlighting selected text. |
| MC.04.02.e010 | Icon & widget style | setting | Appearance settings | Choose the color, background, and behavior of icons and widgets. |
| MC.04.02.e011 | Default | setting | Icon & widget style | Items are opaque, and the background is light or colored, depending on the app. |
| MC.04.02.e012 | Dark | setting | Icon & widget style | Items are opaque and have a darkened appearance. |
| MC.04.02.e013 | Always | setting | Icon & widget style; Dark style | Always show items in Dark style. |
| MC.04.02.e014 | Auto | setting | Icon & widget style; Dark and Clear styles | Switch between Dark and Light depending on system appearance. |
| MC.04.02.e015 | Clear | setting | Icon & widget style | Items are transparent. |
| MC.04.02.e016 | Light | setting | Icon & widget style; Clear style | Choose a light appearance. |
| MC.04.02.e017 | Tinted | setting | Icon & widget style | Items are opaque against a dark background. Icons, text, and graphics use the color selected under Theme in the Appearance pane. |
| MC.04.02.e018 | Folder color | setting | Appearance settings | Choose a color for Finder folder icons. This option appears when Default, Dark, or Clear is selected for Icon & widget style. |
| MC.04.02.e019 | Icon, widget, & folder color | setting | Appearance settings | Choose a color for icons and widgets, and Finder folder icons. This option appears when Tinted is selected for Icon & widget style. |
| MC.04.02.e020 | Sidebar icon size | setting | Appearance settings | Choose small, medium, or large icons for the Finder sidebar and sidebars in apps such as Mail, News, and Photos. |
| MC.04.02.e021 | small | setting | Sidebar icon size | A sidebar icon size option. |
| MC.04.02.e022 | medium | setting | Sidebar icon size | A sidebar icon size option. |
| MC.04.02.e023 | large | setting | Sidebar icon size | A sidebar icon size option. |
| MC.04.02.e024 | Tint window background with wallpaper color | setting | Appearance settings | Allow wallpaper colors to subtly tint certain window areas, such as toolbars or status bars. |
| MC.04.02.e025 | Show scroll bars | setting | Appearance settings | Choose when scroll bars appear in windows with content that doesn’t fit the current window size. |
| MC.04.02.e026 | Automatically based on mouse or trackpad | setting | Show scroll bars | Show scroll bars depending on the mouse or trackpad being used. |
| MC.04.02.e027 | When scrolling | setting | Show scroll bars | Hide scroll bars until scrolling starts. |
| MC.04.02.e028 | Always | setting | Show scroll bars | Always show scroll bars. |
| MC.04.02.e029 | Click in the scroll bar to | setting | Appearance settings | Choose what clicking in a scroll bar does. |
| MC.04.02.e030 | Jump to the next page | setting | Click in the scroll bar to | Clicking in the scroll bar moves content page by page. |
| MC.04.02.e031 | Jump to the spot that’s clicked | setting | Click in the scroll bar to | Clicking moves to the corresponding spot in the content. |
| MC.04.02.e032 | Option | other | Scroll bar instructions | The key to press and hold while clicking the scroll bar to momentarily switch scrolling methods. |
| MC.04.02.e033 | Increase font size and icons | other | Below the settings table | A referenced link for increasing text and icon size. |
| MC.04.02.e034 | Change Night Shift settings | other | Appearance setting description | A referenced link related to the Night Shift schedule. |
| MC.04.02.e035 | Customize the wallpaper on your Mac | other | See also | A related support link. |
| MC.04.02.e036 | Format text in documents with fonts on Mac | other | See also | A related support link. |
| MC.04.02.e037 | Change your Mac display’s resolution | other | See also | A related support link. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.04.02.b001 | Choose Light in Appearance | Buttons, menus, and windows use a light appearance that doesn’t change. | None stated. |
| MC.04.02.b002 | Choose Dark in Appearance | The appearance is darkened; content stands out while windows and controls seem to recede. | The description says this is effective for viewing documents, presentations, photos, movies, webpages, and more. |
| MC.04.02.b003 | Choose Auto in Appearance | The appearance switches from light to dark. | The switch follows the set Night Shift schedule. If no schedule is set, it follows sunrise and sunset times. Auto does not switch until the Mac has been idle for at least a minute. |
| MC.04.02.b004 | Drag the Liquid Glass slider | Adjusts the appearance of app icons from clear at the far left to the heaviest tinting at the far right, with intermediate choices available. | None stated. |
| MC.04.02.b005 | Choose a Color | Sets the color used for buttons, pop-up menus, and other UI controls. | None stated. |
| MC.04.02.b006 | Choose a Text highlight color | Sets the color used to highlight selected text. | None stated. |
| MC.04.02.b007 | Choose Default for Icon & widget style | Items are opaque, and their background is light or colored depending on the app. | Folder color appears as an option when Default is selected. |
| MC.04.02.b008 | Choose Dark for Icon & widget style | Items are opaque and darkened. | Choose Always to always show items in Dark style, or Auto to switch between Dark and Light based on system appearance. Folder color appears as an option. |
| MC.04.02.b009 | Choose Clear for Icon & widget style | Items are transparent. | Choose Light for a light appearance, Dark for a darker appearance, or Auto to switch between Dark and Light based on system appearance. Folder color appears as an option. |
| MC.04.02.b010 | Choose Tinted for Icon & widget style | Items are opaque against a dark background; icons, text, and graphics use the color selected under Theme in the Appearance pane. | The Icon, widget, & folder color option appears when Tinted is selected. |
| MC.04.02.b011 | Choose a Folder color | Sets the color of Finder folder icons. | The option appears when Default, Dark, or Clear is selected for Icon & widget style. |
| MC.04.02.b012 | Choose an Icon, widget, & folder color | Sets the color for icons and widgets, and Finder folder icons. | The option appears when Tinted is selected for Icon & widget style. |
| MC.04.02.b013 | Choose a Sidebar icon size | Changes the size of sidebar icons and sidebar text. | Applies to the Finder sidebar and sidebars in apps such as Mail, News, and Photos. Available sizes are small, medium, and large. |
| MC.04.02.b014 | Enable Tint window background with wallpaper color | Allows wallpaper colors to subtly tint certain window areas, such as toolbars or status bars. | None stated. |
| MC.04.02.b015 | Choose Automatically based on mouse or trackpad for Show scroll bars | Scroll bar visibility depends on the mouse or trackpad being used. | If the device allows gestures, scroll bars are hidden until scrolling starts. Otherwise, they are visible. |
| MC.04.02.b016 | Choose When scrolling for Show scroll bars | Scroll bars remain hidden until scrolling starts. | None stated. |
| MC.04.02.b017 | Choose Always for Show scroll bars | Scroll bars are always shown. | None stated. |
| MC.04.02.b018 | Click in the scroll bar | Content moves page by page. | Applies when Jump to the next page is selected for Click in the scroll bar to. |
| MC.04.02.b019 | Click in the scroll bar | Content moves to the spot corresponding to where the scroll bar was clicked. | Applies when Jump to the spot that’s clicked is selected; clicking at the bottom of the scroll bar goes to the bottom of the document. |
| MC.04.02.b020 | Press and hold Option while clicking the scroll bar | Momentarily switches between scrolling methods while working. | None stated. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.04.02.k001 | Option | Momentarily switch between scrolling methods while working. | Press and hold Option while clicking the scroll bar. |

System Settings paths referenced: Appearance settings; Theme in the Appearance pane; Night Shift settings

#### Independent reports (MC.04.02)

| ID | Kind | Area | Claim | Source |
|---|---|---|---|---|
| MC.04.02.x01 | behavior | windows | The article says macOS 27 Golden Gate's UI changes are moving in a positive direction and that UI and icon changes are still being made during the .0 beta cycle in mid-August. | T4 [Daring Fireball](https://daringfireball.net/linked/2026/08/18/golden-gate-27-window-controls) |
| MC.04.02.x02 | new | windows | In macOS 27 Golden Gate beta 6, the red, yellow, and green window controls have a redesigned look, with more detail in each button and a resemblance to older Mac OS X designs. | T4 [9to5Mac](https://9to5mac.com/2026/08/17/macos-27-golden-gate-beta-6-features-redesigned-traffic-light-window-controls/) |
| MC.04.02.x03 | behavior | windows | The redesigned controls appear on every app window. | T4 [9to5Mac](https://9to5mac.com/2026/08/17/macos-27-golden-gate-beta-6-features-redesigned-traffic-light-window-controls/) |
| MC.04.02.x04 | behavior | windows | The red, yellow, and green window controls manage closing, hiding, and maximizing window size, respectively. | T4 [9to5Mac](https://9to5mac.com/2026/08/17/macos-27-golden-gate-beta-6-features-redesigned-traffic-light-window-controls/) |
| MC.04.02.x05 | changed | windows | macOS 27 Golden Gate fixes window shape consistency; the article says macOS 26 Tahoe had inconsistent window shapes. | T4 [9to5Mac](https://9to5mac.com/2026/08/17/macos-27-golden-gate-beta-6-features-redesigned-traffic-light-window-controls/) |
| MC.04.02.x06 | changed | windows | Beta 6 redesigns the window traffic light buttons with a Liquid Glass, three-dimensional appearance reminiscent of classic Aqua controls. | T4 [iThinkDiff](https://www.ithinkdiff.com/macos-golden-gate-beta-6-wallpapers-traffic-light-buttons/) |
| MC.04.02.x07 | bug | windows | Some light-mode beta testers reported low pixel quality in the redesigned traffic light buttons. | T4 [iThinkDiff](https://www.ithinkdiff.com/macos-golden-gate-beta-6-wallpapers-traffic-light-buttons/) |
| MC.04.02.x08 | changed | Liquid Glass | Beta 6 improves Liquid Glass opacity and adds a slider for controlling transparency. | T4 [iThinkDiff](https://www.ithinkdiff.com/macos-golden-gate-beta-6-wallpapers-traffic-light-buttons/) |
| MC.04.02.x09 | changed | windows | Beta 6 standardizes app toolbars across the system. | T4 [iThinkDiff](https://www.ithinkdiff.com/macos-golden-gate-beta-6-wallpapers-traffic-light-buttons/) |
| MC.04.02.x10 | changed | windows | Beta 6 expands sidebars to the edges of the screen. | T4 [iThinkDiff](https://www.ithinkdiff.com/macos-golden-gate-beta-6-wallpapers-traffic-light-buttons/) |
| MC.04.02.x11 | changed | windows | Beta 6 reduces corner roundness. | T4 [iThinkDiff](https://www.ithinkdiff.com/macos-golden-gate-beta-6-wallpapers-traffic-light-buttons/) |
| MC.04.02.x12 | changed | windows | Window border shapes are consistent across applications, reversing a change introduced in macOS Tahoe. | T3 [Wikipedia](https://en.wikipedia.org/wiki/MacOS_Golden_Gate) |
| MC.04.02.x13 | new | windows | The iPhone Mirroring app supports window resizing. | T3 [Wikipedia](https://en.wikipedia.org/wiki/MacOS_Golden_Gate) |
| MC.04.02.x14 | changed | windows | In Golden Gate, sidebars extend to the top, bottom, and side of the window to which they are anchored. They retain a glassy appearance, and sidebar icons have regained their color. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/3/) |
| MC.04.02.x15 | changed | windows | Golden Gate brings more consistent corner radii to system windows. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/3/) |
| MC.04.02.x16 | changed | windows | Golden Gate uses more pronounced drop shadows to make the separation between overlapping windows easier to see. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/3/) |
| MC.04.02.x17 | changed | Liquid Glass | Golden Gate retains glass-like sidebars but removes the layered effect of sidebars sitting on top of windows. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/3/) |
| MC.04.02.x18 | changed | Liquid Glass | Golden Gate makes Liquid Glass more cohesive and readable, including addressing inconsistencies in how it was applied to Finder’s different view modes. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/2/) |
| MC.04.02.x19 | changed | Liquid Glass | Liquid Glass elements diffuse light more sharply: edges are darker and highlights brighter, making UI elements appear crisper and more readable than in Tahoe. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/2/) |
| MC.04.02.x20 | new | Liquid Glass | System Settings’ Appearance section has a transparency slider and a preview of its effect. The default is at the midpoint; moving the slider left lets more background show through toolbars, while moving it right makes the background more frosted and nearly solid. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/2/) |
| MC.04.02.x21 | changed | Liquid Glass | At the default setting, toolbars diffuse content beneath them more effectively. In Photos, the updated toolbar separates its controls more clearly from scrolling images. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/2/) |
| MC.04.02.x22 | behavior | Liquid Glass | A continuous bar appears beneath toolbar controls automatically when content scrolls under them. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/2/) |
| MC.04.02.x23 | changed | windows | Golden Gate windows have more consistent corners and are generally less rounded than in Tahoe. | T4 [Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/) |
| MC.04.02.x24 | changed | windows | The sidebar sits flush against the left edge of windows, with a darker background and colored icons, rather than floating inset from the edge. | T4 [Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/) |
| MC.04.02.x25 | changed | windows | Toolbars have more shading and legibility than in Tahoe. In Photos, the toolbar is a discrete bar rather than a floating row of buttons. | T4 [Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/) |
| MC.04.02.x26 | behavior | windows | The article reports that toolbars in inactive windows can have more contrast and legibility than those in active windows. | T4 [Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/) |
| MC.04.02.x27 | changed | windows | The red, yellow, and green window controls have Liquid Glass shading. | T4 [Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/) |
| MC.04.02.x28 | new | Liquid Glass | A slider in Settings lets users adjust Liquid Glass transparency from more transparent to more opaque. | T4 [Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/) |
| MC.04.02.x29 | workaround | windows | As a workaround for tiling that has stopped responding, the article suggests restarting the WindowManager process in Terminal with `killall WindowManager`; it says the process restarts automatically. | T4 [Appish](https://appish.app/blog/mac-window-tiling-shortcuts-not-working-fix) |
| MC.04.02.x30 | behavior | windows | Some apps, particularly Electron apps, may resist snapping because they override their window chrome; the article says app updates may help. | T4 [Appish](https://appish.app/blog/mac-window-tiling-shortcuts-not-working-fix) |
| MC.04.02.x31 | workaround | windows | The article suggests logging out and back in as a workaround when Sequoia's window compositing layer needs to be reset. | T4 [Appish](https://appish.app/blog/mac-window-tiling-shortcuts-not-working-fix) |
| MC.04.02.x32 | bug | windows | With multiple monitors, tiled windows may land in the wrong region, overlap, or fail to respect display boundaries; the article recommends checking System Settings → Displays → Arrange… and reconnecting displays one at a time after restarting. | T4 [Appish](https://appish.app/blog/mac-window-tiling-shortcuts-not-working-fix) |
| MC.04.02.x33 | changed | Liquid Glass | Liquid Glass will diffuse complex content behind it more effectively to improve readability. | T4 [Cult of Mac](https://www.cultofmac.com/news/macos-27-new-features-beyond-siri-ai) |
| MC.04.02.x34 | new | Liquid Glass | A slider will let users adjust Liquid Glass’s tinted effect. | T4 [Cult of Mac](https://www.cultofmac.com/news/macos-27-new-features-beyond-siri-ai) |
| MC.04.02.x35 | changed | Liquid Glass | System icons have been revamped to stand out more. | T4 [Cult of Mac](https://www.cultofmac.com/news/macos-27-new-features-beyond-siri-ai) |
| MC.04.02.x36 | changed | windows | Apps will have a consistent window corner radius, including apps whose developers do not update them for macOS 27. | T4 [Cult of Mac](https://www.cultofmac.com/news/macos-27-new-features-beyond-siri-ai) |
| MC.04.02.x37 | changed | windows | The active app’s sidebar icons will be colored. | T4 [Cult of Mac](https://www.cultofmac.com/news/macos-27-new-features-beyond-siri-ai) |
| MC.04.02.x38 | changed | windows | A sidebar will expand to the edge of its window. | T4 [Cult of Mac](https://www.cultofmac.com/news/macos-27-new-features-beyond-siri-ai) |
| MC.04.02.x39 | changed | windows | Toolbars will be unified. | T4 [Cult of Mac](https://www.cultofmac.com/news/macos-27-new-features-beyond-siri-ai) |
| MC.04.02.x40 | behavior | windows | Windows within a Stage Manager set can be moved, resized, and overlapped; Stage Manager does not tile them. | T4 [Lets Suite](https://www.lets-suite.com/blog/how-to-use-stage-manager-mac) |
| MC.04.02.x41 | behavior | windows | The article says Stage Manager is less suited to workflows that need many windows visible at once, because windows off the stage are out of sight. | T4 [Lets Suite](https://www.lets-suite.com/blog/how-to-use-stage-manager-mac) |
| MC.04.02.x42 | changed | windows | Apple says window positioning is more consistent across external displays, so apps and Spaces move around less. The article says the public betas appear to target windows landing on the wrong screen or at the wrong size when connecting or disconnecting a display; Apple has not described the mechanics, and the improvement is not a guarantee that windows never move. | T4 [Lets Suite](https://www.lets-suite.com/blog/macos-27-window-management) |
| MC.04.02.x43 | behavior | windows | The article says windows that were on a display that is no longer connected still have to go somewhere. | T4 [Lets Suite](https://www.lets-suite.com/blog/macos-27-window-management) |
| MC.04.02.x44 | changed | windows | App windows share the same corner radius in macOS 27, and their corners are less dramatically rounded than in Tahoe. | T4 [Lets Suite](https://www.lets-suite.com/blog/macos-27-window-management) |
| MC.04.02.x45 | changed | windows | Window shadows have changed and, together with Liquid Glass revisions, are intended to make the active window more apparent. | T4 [Lets Suite](https://www.lets-suite.com/blog/macos-27-window-management) |
| MC.04.02.x46 | new | Liquid Glass | A control lets users make Liquid Glass clearer, allowing more of the background through, or more opaque for legibility. | T4 [Lets Suite](https://www.lets-suite.com/blog/macos-27-window-management) |
| MC.04.02.x47 | changed | windows | Sidebars are edge-to-edge rather than floating, and toolbars are more uniform. | T4 [Lets Suite](https://www.lets-suite.com/blog/macos-27-window-management) |
| MC.04.02.x48 | changed | windows | iPhone Mirroring windows can be resized larger than before. | T4 [Lets Suite](https://www.lets-suite.com/blog/macos-27-window-management) |
| MC.04.02.x49 | changed | windows | Sidecar has better touch support when an iPad is used as a display. | T4 [Lets Suite](https://www.lets-suite.com/blog/macos-27-window-management) |
| MC.04.02.x50 | behavior | windows | macOS still has no built-in saved window layouts or “restore my setup” feature. | T4 [Lets Suite](https://www.lets-suite.com/blog/macos-27-window-management) |
| MC.04.02.x51 | behavior | windows | Third-party window managers rely on Accessibility permission, which the article says Golden Gate does not change. If an app stops responding after upgrading, it recommends checking “System Settings” → “Privacy & Security” → “Accessibility.” | T4 [Lets Suite](https://www.lets-suite.com/blog/macos-27-window-management) |
| MC.04.02.x52 | behavior | windows | Close System Settings using the red Close button in the top-left corner of the window or the Cmd + W keyboard shortcut. | T4 [AbilityNet](https://api.abilitynet.org.uk/how-to-enable-the-accessibility-options-on-the-login-window-in-macos-27-golden-gate) |
| MC.04.02.x53 | measurement | windows | The pull request reports that Notification Center's panel appears as a display-sized window at layer 21 about 166 ms after a click; Control Centre's panel appears as a 656×964 window named “Control Center” at layer 101 about 177 ms after an Accessibility press. | T2 [GitHub jordanbaird/Ice](https://github.com/jordanbaird/Ice/pull/995) |
| MC.04.02.x54 | behavior | windows | Entering or leaving full screen swaps the menu bar used to draw items; a follow-up reports that concealment must be reapplied so Ice's own item is available on the newly shown bar. | T2 [GitHub jordanbaird/Ice](https://github.com/jordanbaird/Ice/pull/995) |
| MC.04.02.x55 | new | Liquid Glass | System Settings adds a slider to adjust Liquid Glass transparency from ultra-clear to fully tinted. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MC.04.02.x56 | changed | Liquid Glass | Apple has improved contrast and window shadows, standardized macOS toolbars, and made sidebars extend to the edges of app windows. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MC.04.02.x57 | new | windows | iPhone Mirroring windows can be resized by dragging a corner; supported apps can show more information as the window expands. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MC.04.02.x58 | new | windows | Golden Gate adds resolution and refresh-rate choices for compatible external displays, including 5K at 120Hz on certain ultrawide screens. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MC.04.02.x59 | behavior | windows | macOS 27 will give all apps a consistent corner radius, including third-party apps. The change is automatic and requires no developer work. | T4 [Cult of Mac](https://www.cultofmac.com/news/liquid-glass-changes-ios-27-macos-27) |
| MC.04.02.x60 | changed | windows | Sidebars will extend to the edge of the window. | T4 [Cult of Mac](https://www.cultofmac.com/news/liquid-glass-changes-ios-27-macos-27) |
| MC.04.02.x61 | changed | windows | Sidebar icons will be colored only for the active app. | T4 [Cult of Mac](https://www.cultofmac.com/news/liquid-glass-changes-ios-27-macos-27) |
| MC.04.02.x62 | new | Liquid Glass | A slider in Settings will let users adjust Liquid Glass from ultra clear to fully tinted across apps. The tinted end increases contrast and improves readability. | T4 [Cult of Mac](https://www.cultofmac.com/news/liquid-glass-changes-ios-27-macos-27) |
| MC.04.02.x63 | changed | other | App icons will incorporate additional layers of Liquid Glass artwork. Apple says the layers’ refraction adds character and makes icons appear sharper and more defined. | T4 [Cult of Mac](https://www.cultofmac.com/news/liquid-glass-changes-ios-27-macos-27) |
| MC.04.02.x64 | changed | windows | Apps will have a uniform toolbar across the top, with a frosted look intended to improve legibility and distinguish toolbar buttons from content below. | T4 [Cult of Mac](https://www.cultofmac.com/news/liquid-glass-changes-ios-27-macos-27) |
| MC.04.02.x65 | behavior | Liquid Glass | Most design improvements will apply automatically to apps that already use the Liquid Glass framework. | T4 [Cult of Mac](https://www.cultofmac.com/news/liquid-glass-changes-ios-27-macos-27) |
| MC.04.02.x66 | behavior | Liquid Glass | On Mac, the Liquid Glass appearance slider is in System Settings, in the Appearance section. | T4 [AppleInsider](https://appleinsider.com/articles/26/06/09/dont-get-excited-for-big-liquid-glass-changes-in-macos-27-because-they-arent-there) |
| MC.04.02.x67 | behavior | Liquid Glass | The slider controls how transparent or translucent Liquid Glass appears. It defaults to the center and can be dragged left or right. | T4 [AppleInsider](https://appleinsider.com/articles/26/06/09/dont-get-excited-for-big-liquid-glass-changes-in-macos-27-because-they-arent-there) |
| MC.04.02.x68 | behavior | Liquid Glass | An example screen shows overlays, including a share button that can appear atop a window, against various background images. | T4 [AppleInsider](https://appleinsider.com/articles/26/06/09/dont-get-excited-for-big-liquid-glass-changes-in-macos-27-because-they-arent-there) |
| MC.04.02.x69 | behavior | Liquid Glass | At one end of the slider, elements such as search bars are closer to opaque; at the other, they are closer to translucent. | T4 [AppleInsider](https://appleinsider.com/articles/26/06/09/dont-get-excited-for-big-liquid-glass-changes-in-macos-27-because-they-arent-there) |
| MC.04.02.x70 | measurement | Liquid Glass | The visual difference from changing the slider is subtle on Mac, and less noticeable than on iPhone, according to the article. | T4 [AppleInsider](https://appleinsider.com/articles/26/06/09/dont-get-excited-for-big-liquid-glass-changes-in-macos-27-because-they-arent-there) |
| MC.04.02.x71 | new | windows | In macOS Golden Gate, window corners have a uniform radius, including in third-party apps. | T4 [AppleInsider](https://appleinsider.com/articles/26/06/09/dont-get-excited-for-big-liquid-glass-changes-in-macos-27-because-they-arent-there) |
| MC.04.02.x72 | behavior | windows | Tab-W opens the Window Chooser, and Tab-A opens the Application Chooser. | T4 [Lets Suite](https://www.lets-suite.com/blog/full-keyboard-access-mac) |
| MC.04.02.x73 | behavior | windows | Full Keyboard Access can reach a window's toolbar, a sheet, and a sidebar, as well as move focus back into window content. | T4 [Lets Suite](https://www.lets-suite.com/blog/full-keyboard-access-mac) |
| MC.04.02.x74 | behavior | windows | The article says the Window Chooser has no fast most-recently-used order. | T4 [Lets Suite](https://www.lets-suite.com/blog/full-keyboard-access-mac) |
| MC.04.02.x75 | new | Liquid Glass | The post reports a new Liquid Glass appearance, with system UI reflections rendering in HDR and a new effect slider. | T5 [MacRumors Forums](https://forums.macrumors.com/threads/macos-27-all-the-little-things.2483520/) |
| MC.04.02.x76 | new | windows | The post reports a consistent new corner radius for all windows. | T5 [MacRumors Forums](https://forums.macrumors.com/threads/macos-27-all-the-little-things.2483520/) |
| MC.04.02.x77 | new | windows | The post reports that inactive windows are dimmed. | T5 [MacRumors Forums](https://forums.macrumors.com/threads/macos-27-all-the-little-things.2483520/) |
| MC.04.02.x78 | new | windows | The post reports a new hand cursor reminiscent of Mickey Mouse gloves. | T5 [MacRumors Forums](https://forums.macrumors.com/threads/macos-27-all-the-little-things.2483520/) |
| MC.04.02.x79 | behavior | windows | The wallpaper becomes darker when tiled windows fill the screen. | T5 [MacRumors Forums](https://forums.macrumors.com/threads/macos-27-all-the-little-things.2483520/) |
| MC.04.02.x80 | new | windows | Opening the Siri AI app brings up a dedicated chat window. | T4 [Tom's Guide](https://www.tomsguide.com/computing/macos/macos-golden-gate-review) |
| MC.04.02.x81 | new | Liquid Glass | A slider lets users adjust the opacity of the Liquid Glass effect. | T4 [Tom's Guide](https://www.tomsguide.com/computing/macos/macos-golden-gate-review) |
| MC.04.02.x82 | behavior | Liquid Glass | Complex background visuals appear more diffused, which the article says improves text readability. | T4 [Tom's Guide](https://www.tomsguide.com/computing/macos/macos-golden-gate-review) |
| MC.04.02.x83 | new | Liquid Glass | Icon edges are more pronounced so icons stand out more from the background. | T4 [Tom's Guide](https://www.tomsguide.com/computing/macos/macos-golden-gate-review) |
| MC.04.02.x84 | new | windows | App windows have a uniform corner radius. | T4 [Tom's Guide](https://www.tomsguide.com/computing/macos/macos-golden-gate-review) |
| MC.04.02.x85 | changed | windows | When linked on macOS 27.0 or later, NSTitlebarAccessoryViewController is allowed to draw outside its bounds by default, supporting effects such as shadows and interactive glass effects. Accessories may be clipped during reveal animations or while hidden. | T4 [BetaProfile](https://betaprofile.dev/macos-27-rc/) |
| MC.04.02.x86 | bug | windows | Fixed an issue where a window could be lost when exiting full-screen mode through Mission Control. | T4 [BetaProfile](https://betaprofile.dev/macos-27-rc/) |
| MC.04.02.x87 | new | Liquid Glass | A new Liquid Glass slider lets users adjust how transparent or opaque the effect appears. | T4 [9to5Mac](https://9to5mac.com/2026/09/14/macos-27-golden-gate-now-available-here-is-everything-new/) |
| MC.04.02.x88 | behavior | Liquid Glass | The Liquid Glass appearance demo has a Clarus the Dogcow easter egg that appears when users scroll the demo pane all the way down and keep going. | T4 [9to5Mac](https://9to5mac.com/2026/09/14/macos-27-golden-gate-now-available-here-is-everything-new/) |
| MC.04.02.x89 | changed | Liquid Glass | App icons have been revamped with additional layers for depth and detail, across light, dark, tinted, and all-clear system appearance settings. | T4 [9to5Mac](https://9to5mac.com/2026/09/14/macos-27-golden-gate-now-available-here-is-everything-new/) |
| MC.04.02.x90 | changed | windows | System windows and apps now use a standardized corner radius rather than a radius that varies with the UI elements in a window’s menu bar. | T4 [9to5Mac](https://9to5mac.com/2026/09/14/macos-27-golden-gate-now-available-here-is-everything-new/) |
| MC.04.02.x91 | changed | windows | The system adds edge-to-edge sidebars and makes active windows more distinct. | T4 [9to5Mac](https://9to5mac.com/2026/09/14/macos-27-golden-gate-now-available-here-is-everything-new/) |
| MC.04.02.x92 | changed | windows | Window position consistency and persistence across external displays have been improved. | T4 [9to5Mac](https://9to5mac.com/2026/09/14/macos-27-golden-gate-now-available-here-is-everything-new/) |
| MC.04.02.x93 | changed | windows | The iPhone Mirroring window can be resized to different aspect ratios. | T4 [9to5Mac](https://9to5mac.com/2026/09/14/macos-27-golden-gate-now-available-here-is-everything-new/) |

### MC.04.03 Traffic-light controls (27 redesign)

Status: **PARTIAL**. Redesign corroborated by 6 independent sources; no Apple T1 description (GAP-B5-01, supersedes Part 1 GAP-007).

Also covered by: [Move and arrange app windows on Mac](https://support.apple.com/guide/mac-help/mchlp2469/27/mac/27), [Windows](https://developer.apple.com/design/human-interface-guidelines/windows), [Modernize your AppKit app](https://developer.apple.com/videos/play/wwdc2026/289/), [macOS 27 Golden Gate Release Notes](https://developer.apple.com/documentation/macos-release-notes/macos-27-release-notes)


#### Independent reports (MC.04.03)

| ID | Kind | Area | Claim | Source |
|---|---|---|---|---|
| MC.04.03.x01 | new | traffic lights | In macOS 27 Golden Gate beta 6, the traffic light window controls have a redesigned appearance, with more detail in each button and a look resembling older Mac OS X systems. | T4 [Daring Fireball](https://daringfireball.net/linked/2026/08/18/golden-gate-27-window-controls) |
| MC.04.03.x02 | new | traffic lights | In the sixth beta of macOS 27, the three traffic-light buttons in Mac windows took on a Liquid Glass appearance with subtle reflections, making them look less flat. (translated) | T4 [iCulture](https://www.iculture.nl/nieuws/mac-stoplicht-knoppen-liquid-glass-macos-27/) |
| MC.04.03.x03 | behavior | traffic lights | In a window that isn’t in the foreground, the traffic-light buttons turn gray; white reflections are visible along the top and bottom. (translated) | T4 [iCulture](https://www.iculture.nl/nieuws/mac-stoplicht-knoppen-liquid-glass-macos-27/) |
| MC.04.03.x04 | behavior | traffic lights | When you click and drag the traffic-light buttons with the cursor in macOS 27, they stretch and then bounce back to their original shape. (translated) | T4 [iCulture](https://www.iculture.nl/nieuws/mac-stoplicht-knoppen-liquid-glass-macos-27/) |
| MC.04.03.x05 | behavior | other | The article describes some Liquid Glass design moments in macOS 27 as Aqua-like. | T4 [9to5Mac](https://9to5mac.com/2026/08/17/macos-27-golden-gate-beta-6-features-redesigned-traffic-light-window-controls/) |
| MC.04.03.x06 | changed | other | The Preview app icon was updated to match the redesigned version introduced in iOS 27 beta 5. | T4 [9to5Mac](https://9to5mac.com/2026/08/17/macos-27-golden-gate-beta-6-features-redesigned-traffic-light-window-controls/) |
| MC.04.03.x07 | new | other | Beta 6 adds animated “Golden Gate Day” and “Golden Gate Evening” wallpapers. They animate when the user unlocks the Mac and can also be set as screen savers. | T4 [iThinkDiff](https://www.ithinkdiff.com/macos-golden-gate-beta-6-wallpapers-traffic-light-buttons/) |
| MC.04.03.x08 | behavior | other | As screen savers, the new wallpapers show cars slowly driving across the bridge. | T4 [iThinkDiff](https://www.ithinkdiff.com/macos-golden-gate-beta-6-wallpapers-traffic-light-buttons/) |
| MC.04.03.x09 | changed | traffic lights | The traffic light window control buttons were redesigned with an Aqua-like Liquid Glass appearance, similar to versions before OS X Yosemite. | T3 [Wikipedia](https://en.wikipedia.org/wiki/MacOS_Golden_Gate) |
| MC.04.03.x10 | changed | other | The mouseover cursor again uses a glove icon, with a different design from earlier versions. | T3 [Wikipedia](https://en.wikipedia.org/wiki/MacOS_Golden_Gate) |
| MC.04.03.x11 | changed | other | Many animations changed, and buttons now animate similarly to those in iOS and iPadOS. | T3 [Wikipedia](https://en.wikipedia.org/wiki/MacOS_Golden_Gate) |
| MC.04.03.x12 | changed | other | Top bars in apps now resemble the menu bars from iPadOS. | T3 [Wikipedia](https://en.wikipedia.org/wiki/MacOS_Golden_Gate) |
| MC.04.03.x13 | changed | traffic lights | The window traffic light buttons have a Liquid Glass appearance and bounce when users click and drag them before releasing. | T4 [9to5Mac](https://9to5mac.com/2026/09/14/macos-27-golden-gate-now-available-here-is-everything-new/) |

### MC.04.04 Full screen

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).


#### Use apps in full screen on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchl9c21d2be/27/mac/27) · macOS 27 text present: no

Apps that support full-screen mode can fill the screen. The page explains how to enter and exit full screen, show or hide the menu bar and Dock, switch between full-screen apps, and use Split View.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.04.04.e001 | green button | button | Top-left corner of an app window | Click to enter full screen. Hover over it to access full-screen options or exit full screen. |
| MC.04.04.e002 | Full Screen | menu item | Menu shown when the pointer is held over the green button | Opens the full-screen options. |
| MC.04.04.e003 | Entire Screen | menu item | Full Screen menu | Enters full-screen mode. |
| MC.04.04.e004 | menu bar | view | Top of the screen | Can be shown or hidden in full screen by moving the pointer to or away from the top of the screen. |
| MC.04.04.e005 | Automatically hide and show the menu bar | setting | Referenced in the note about menu bar visibility | The page says this setting can be changed to Never to keep the menu bar visible in full screen. |
| MC.04.04.e006 | Never | setting | Value for “Automatically hide and show the menu bar” | Keeps the menu bar visible in full screen. |
| MC.04.04.e007 | Change Menu Bar settings | other | Link in the note about keeping the menu bar visible | Referenced for information about changing the menu bar setting. |
| MC.04.04.e008 | Dock | view | At the Dock’s location on the screen | Can be shown or hidden in full screen by moving the pointer to or away from its location. |
| MC.04.04.e009 | Exit Full Screen | menu item | Menu shown when the pointer is held over the green button | Stops using the app in full screen. |
| MC.04.04.e010 | Maximize or minimize app windows | other | See also link | Referenced for information about maximizing a window without entering full screen. |
| MC.04.04.e011 | Mission Control | view | Entered using Control-Up Arrow or an upward trackpad swipe | Shows windows and spaces; the page describes using it to set up Split View. |
| MC.04.04.e012 | Spaces bar | view | In Mission Control | The page says to drag a window or app thumbnail onto a full-screen app thumbnail or another thumbnail here. |
| MC.04.04.e013 | Split View | view | Selected by clicking the Split View thumbnail | Lets the user use another app alongside an app that is already full screen. |
| MC.04.04.e014 | Split View thumbnail | other | In Mission Control after dragging a window onto the full-screen app thumbnail | Click to enter Split View. |
| MC.04.04.e015 | View open windows and spaces in Mission Control on Mac | other | See also | Related link. |
| MC.04.04.e016 | Work in multiple spaces on Mac | other | See also | Related link. |
| MC.04.04.e017 | Move and arrange app windows on Mac | other | See also | Related link. |
| MC.04.04.e018 | Tile windows on Mac | other | See also | Related link. |
| MC.04.04.e019 | Mac window tiling icons & keyboard shortcuts | other | See also | Related link. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.04.04.b001 | Click the green button in the top-left corner of a window. | The app enters full-screen mode. | The app supports full-screen mode. |
| MC.04.04.b002 | Hold the pointer over the green button, then choose Full Screen > Entire Screen. | The app enters full-screen mode. | The app supports full-screen mode. |
| MC.04.04.b003 | Move the pointer to or away from the top of the screen. | The menu bar is shown or hidden. | The app is in full-screen mode. The “Automatically hide and show the menu bar” setting can be set to Never to keep the menu bar visible. |
| MC.04.04.b004 | Move the pointer to or away from the Dock’s location. | The Dock is shown or hidden. | The app is in full-screen mode. |
| MC.04.04.b005 | Swipe left or right on the trackpad with three or four fingers. | Move between other apps in full screen. | The finger count depends on the trackpad settings. |
| MC.04.04.b006 | Hold the pointer over the green button, then choose Exit Full Screen. | Stop using the app in full screen. | The app is in full-screen mode. |
| MC.04.04.b007 | Maximize the window. | The window expands while the menu bar and Dock remain visible. | This is an alternative to using full-screen mode. |
| MC.04.04.b008 | Enter Mission Control, drag a window onto the thumbnail of the full-screen app in the Spaces bar, then click the Split View thumbnail. | Use the selected app in Split View alongside the full-screen app. | The page describes this as a way to quickly choose another app while an app is full screen. |
| MC.04.04.b009 | Drag an app’s thumbnail onto another in the Spaces bar. | The apps can be arranged in Split View. | The user is in Mission Control. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.04.04.k001 | Control-Up Arrow | Enter Mission Control. | The page describes using this to choose another app for Split View while an app is full screen. |
| MC.04.04.k002 | Swipe up with three or four fingers | Enter Mission Control. | Trackpad gesture; the page does not specify a different finger count for this gesture. |
| MC.04.04.k003 | Swipe left or right with three or four fingers | Move between other apps in full screen. | Trackpad gesture; finger count depends on trackpad settings. |

System Settings paths referenced: “Automatically hide and show the menu bar” — the page refers to “Change Menu Bar settings” but does not give a System Settings path.

#### Going full screen

T1 · HIG (developer guidance) · [Apple HIG](https://developer.apple.com/design/human-interface-guidelines/going-full-screen) · macOS 27 text present: no

Guidance for offering and designing full-screen experiences, including macOS-specific recommendations to use system full-screen support, preserve access to the Dock, and let people choose when to enter or exit full screen.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.04.04.e020 | Enter Full Screen | button | A window | The system-provided window button people can use to enter full-screen mode. |
| MC.04.04.e021 | View | menu | The macOS menu bar | The menu containing an item people can use to enter full-screen mode. |
| MC.04.04.e022 | Dock | view | iPadOS and macOS | People should be able to reveal the Dock while an app is in full-screen mode; macOS apps can hide it entirely to help prevent accidental reveals during gameplay. |
| MC.04.04.e023 | toolbars and navigation controls | control | An app interface | May be temporarily hidden when content is the primary focus, but should remain visible when essential for navigation or tasks. |
| MC.04.04.e024 | camera housing | other | The top-center area of some Mac displays | The system’s full-screen support automatically accommodates this area. |

| ID | Context | Guideline | Conditions |
|---|---|---|---|
| MC.04.04.b010 | When deciding whether to support full-screen mode | Consider offering it for experiences such as games, media viewing, or in-depth tasks that benefit from a distraction-free environment. | General guidance; full-screen mode is described for iPhone, iPad, and Mac. |
| MC.04.04.b011 | When a window is larger in full-screen mode | Keep essential content prominent and use the extra space; any layout adjustments should be subtle and should not programmatically resize the window. | macOS-relevant general guidance. |
| MC.04.04.b012 | When someone is using a full-screen experience | Keep essential features and controls available without requiring them to exit full screen. | For example, playback controls should persist or be easy to reveal. |
| MC.04.04.b013 | When an iPadOS or macOS app is in full-screen mode | Let people reveal the Dock. | Except in games; macOS games may hide the Dock entirely. |
| MC.04.04.b014 | When someone switches away from a full-screen experience and later returns | Help them resume where they left off; a game or slideshow should pause automatically when they leave. | General guidance. |
| MC.04.04.b015 | When someone switches to another experience or finishes an absorbing activity | Do not automatically end full-screen mode; let people choose when to exit. | General guidance. |
| MC.04.04.b016 | When content is the primary focus | Consider temporarily hiding toolbars and navigation controls, and let people restore them with a familiar gesture or action such as tapping, swiping down, or moving the cursor to the top of the screen. | Keep controls visible when they are essential for navigation or tasks. |
| MC.04.04.b017 | When providing full-screen mode on macOS | Use the system-provided full-screen experience. | This supports different contexts and automatically accommodates areas such as a Mac camera housing. |
| MC.04.04.b018 | When a game enters full-screen mode on macOS | Do not change the display mode. | People are expected to control their display mode. |
| MC.04.04.b019 | When people enter full-screen mode on macOS | Let them choose when to enter, preferably using the window’s Enter Full Screen button, the View menu item, or Control-Command-F; avoid a custom menu of window modes. | A game may also provide a custom toggle to turn full-screen mode on and off. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.04.04.k004 | Control-Command-F | Enter full-screen mode | macOS window |

API names: `preferredEdgesDeferringSystemGestures (SwiftUI)`, `preferredEdgesDeferringSystemGestures (UIKit)`, `hideDock (AppKit)`, `toggle`, `NSScreen`, `NSWindow`, `fullCover(item:onDismiss:content:)`, `preferredScreenEdgesDeferringSystemGestures`

#### Independent reports (MC.04.04)

| ID | Kind | Area | Claim | Source |
|---|---|---|---|---|
| MC.04.04.x01 | bug | full screen | The author says switching to a full-screen application sometimes results in a completely black screen. | T5 [Reddit r/MacOS](https://www.reddit.com/r/MacOS/comments/1wlax16/mission_control_expose_is_broken_in_macos_27/) |
| MC.04.04.x02 | behavior | full screen | The article says another app being in full-screen mode can suppress the layout picker in adjacent windows. | T4 [Appish](https://appish.app/blog/mac-window-tiling-shortcuts-not-working-fix) |

### MC.04.05 Window tiling & Move & Resize

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).

Also covered by: [Move and arrange app windows on Mac](https://support.apple.com/guide/mac-help/mchlp2469/27/mac/27)


#### Tile windows on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlef287e5d/27/mac/27) · macOS 27 text present: no

The page describes ways to tile windows on Mac using screen-edge dragging, Option-dragging, the green window button, Window menu options, and keyboard shortcuts. It references Desktop & Dock settings but does not specify a macOS version or describe macOS 27 changes.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.04.05.e001 | green button | button | Upper-left corner of each window | Opens window layout options when the pointer is held over it. |
| MC.04.05.e002 | Window | menu | Menu bar | Contains window tiling and arrangement options. |
| MC.04.05.e003 | Move & Resize | menu item | Green button options and Window menu | Contains layouts for tiling the active window; in the Window menu, it also contains options for arranging multiple windows. |
| MC.04.05.e004 | Fill & Arrange | menu item | Green button options | Contains layouts for tiling multiple windows. |
| MC.04.05.e005 | Full Screen | menu item | Green button options | Hover over it to choose a full-screen option. |
| MC.04.05.e006 | Halves | menu item | Window > Move & Resize | A group of layouts for tiling the active window. |
| MC.04.05.e007 | Quarters | menu item | Window > Move & Resize | A group of layouts for tiling the active window. |
| MC.04.05.e008 | Arrange | menu item | Window > Move & Resize | A group of layouts for tiling multiple windows; additional layouts are available by pressing Option. |
| MC.04.05.e009 | Center | menu item | Window menu | Centers the active window on the desktop. |
| MC.04.05.e010 | Fill | menu item | Window menu | Fills the desktop with the active window. |
| MC.04.05.e011 | Full Screen Tile | menu item | Window menu | Hover over it to choose a full-screen tiling option. |
| MC.04.05.e012 | Mission Control | other | Screen | Opens when a window is dragged all the way through the top edge of the screen, unless the referenced setting is turned off. |
| MC.04.05.e013 | Drag windows to top of screen to enter Mission Control | setting | Desktop & Dock settings | Controls whether dragging a window all the way through the top edge of the screen opens Mission Control. |
| MC.04.05.e014 | Desktop & Dock | panel | System Settings | The settings area referenced for the Mission Control drag setting. |
| MC.04.05.e015 | Change window tiling settings | other | Referenced support topic | Referenced for turning the window tiling feature on or off; the specific setting name and path are not given. |
| MC.04.05.e016 | Option | other | Keyboard | The key used while dragging to tile a window, and to reveal additional layout options in the described interfaces. |
| MC.04.05.e017 | highlighted area | other | Screen | Appears while dragging a window toward a tiling position. |
| MC.04.05.e018 | edge of the screen | other | Screen perimeter | The left, right, or any corner; the top edge is also mentioned. Dragging to the menu bar fills the desktop. |
| MC.04.05.e019 | menu bar | other | Top of the screen | Dragging a window to it fills the desktop. |
| MC.04.05.e020 | Full Screen | menu item | Window menu | The Window menu contains a separate Full Screen Tile option for choosing a full-screen tile. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.04.05.b001 | Drag a window to the left or right edge, or any corner of the screen, and release when the highlighted area appears. | The window snaps into position. | The pointer must reach the edge of the screen; window tiling can be turned on or off. |
| MC.04.05.b002 | Drag a window to the menu bar at the top of the screen. | The window fills the desktop. | Window tiling can be turned on or off. |
| MC.04.05.b003 | Drag a window all the way through the top edge of the screen. | Mission Control opens. | Turn off “Drag windows to top of screen to enter Mission Control” in Desktop & Dock settings to prevent this. |
| MC.04.05.b004 | Press and hold Option while dragging a window toward the top, left, or right edge, or any corner; release it in the highlighted area. | The window tiles in the chosen area. | The page says this method is for faster window tiling; window tiling can be turned on or off. |
| MC.04.05.b005 | Hold the pointer over the green button in the upper-left corner of the active window. | Window layout options are offered. | The green button is on each window. |
| MC.04.05.b006 | Press Option while viewing the green button options. | Additional layouts are shown. | The pointer is held over the green button. |
| MC.04.05.b007 | Click a layout below Move & Resize in the green button options. | The active window tiles. | The green button options are open. |
| MC.04.05.b008 | Click a layout below Fill & Arrange in the green button options. | Multiple windows tile. | The green button options are open. |
| MC.04.05.b009 | Hold the pointer over Full Screen in the green button options, then choose an option. | A full-screen tiling option is applied. | The green button options are open. |
| MC.04.05.b010 | Press Option, then click the center control described in the green button instructions. | The active window is centered. | The page does not show a text label for the control; the source text ends after “then click.” |
| MC.04.05.b011 | Click the fill control described in the green button instructions. | The active window fills the desktop. | The page does not show a text label for the control. |
| MC.04.05.b012 | Make the window to be tiled active, then choose Window in the menu bar and hold the pointer over Move & Resize; choose a layout below Halves or Quarters. | The active window tiles. | The window to be tiled must be active. |
| MC.04.05.b013 | Choose Window in the menu bar, hold the pointer over Move & Resize, and choose a layout below Arrange. | Multiple windows tile. | Press Option to see additional layouts below Arrange. |
| MC.04.05.b014 | Choose Center from the Window menu. | The active window is centered on the desktop. | The active window is the window to be centered. |
| MC.04.05.b015 | Choose Fill from the Window menu. | The active window fills the desktop. | The active window is the window to be filled. |
| MC.04.05.b016 | Hold the pointer over Full Screen Tile in the Window menu, then choose an option. | A full-screen tiling option is applied. | The Window menu is open. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.04.05.k001 | Option | Hold while dragging a window toward the top, left, or right edge, or a corner, to tile it in the highlighted area. | Window tiling by dragging. |
| MC.04.05.k002 | Option | Press to see additional layouts. | Green button options and layouts below Arrange in Window > Move & Resize. |
| MC.04.05.k003 | Option | Press, then click the center control to center the active window. | Green button instructions; the page does not show the control’s label. |

System Settings paths referenced: System Settings > Desktop & Dock; “Drag windows to top of screen to enter Mission Control”; Change window tiling settings (referenced; specific System Settings path not stated)

#### View and open apps on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mh35840/27/mac/27) · macOS 27 text present: no

Spotlight is described as the place to search, browse, and open apps on Mac. The page covers opening apps, browsing and filtering apps, adding apps to Spotlight, and accessing games.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.04.05.e021 | Spotlight | window | On Mac; opened from the Dock or menu bar | The central location to search, browse, and open apps. |
| MC.04.05.e022 | Dock | other | On Mac | Can be clicked to access Spotlight; apps can also be dragged here from the apps browsing window. |
| MC.04.05.e023 | menu bar | other | At the top of the Mac screen | Can be clicked to access Spotlight. |
| MC.04.05.e024 | search field | control | In Spotlight | Used to enter an app name. |
| MC.04.05.e025 | app icon | icon | In Spotlight | Can be clicked to open an app; may be dimmed when Screen Time restrictions apply. |
| MC.04.05.e026 | hourglass icon | icon | Beside an app in Spotlight when Screen Time restrictions apply | Shown during downtime or after reaching an app time limit. |
| MC.04.05.e027 | Screen Time settings | setting | Referenced in the app restriction description | Used to set app time limits. |
| MC.04.05.e028 | apps window | window | In Spotlight | Used to browse apps, categories, and views. |
| MC.04.05.e029 | Productivity & Finance | other | Along the top of the apps window | An example of an app category. |
| MC.04.05.e030 | Grid | other | View choices for the apps window | A view option. |
| MC.04.05.e031 | List | other | View choices for the apps window | A view option. |
| MC.04.05.e032 | iPhone apps | other | View choices for the apps window | An option to show iPhone apps. |
| MC.04.05.e033 | Arcade | other | A category in the apps window | Games downloaded from Apple Games are added to this category. |
| MC.04.05.e034 | Apple Games | other | Referenced in the description of the Arcade category | Games downloaded from here are added to Arcade in the apps window. |
| MC.04.05.e035 | Applications folder | other | In the Finder | Drag an app not downloaded from the App Store here to have it shown in Spotlight. |
| MC.04.05.e036 | Finder | other | On Mac | Contains the Applications folder. |
| MC.04.05.e037 | App Store | other | On Mac | Apps downloaded from the App Store automatically appear in Spotlight. |
| MC.04.05.e038 | desktop | other | On Mac | An app can be dragged here for easy access. |
| MC.04.05.e039 | Esc | other | Keyboard | Closes Spotlight without opening an app. |
| MC.04.05.e040 | Return | other | Keyboard | Pressed after entering an app name in the search field. |
| MC.04.05.e041 | arrow key | other | Keyboard | Moves to another app while navigating Spotlight. |
| MC.04.05.e042 | game controller | other | Used with the apps window | A compatible game controller can be used to access and navigate Arcade. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.04.05.b017 | Click Dock, or click the menu bar and then click the omitted control shown in the source. | Open Spotlight. | The source leaves the control after “then click” unspecified. |
| MC.04.05.b018 | Enter an app name in the search field, then press Return. | Search for the app in Spotlight. | — |
| MC.04.05.b019 | Click an app icon. | Open the app. | The app is shown in Spotlight. |
| MC.04.05.b020 | Click outside the Spotlight window or press Esc. | Close Spotlight without opening an app. | — |
| MC.04.05.b021 | During downtime or after reaching an app time limit set in Screen Time settings. | App icons are dimmed and an hourglass icon is shown. | The restriction applies to the apps. |
| MC.04.05.b022 | Click Dock, or click the menu bar and press ⌘-1. | Navigate in the apps window of Spotlight. | The source describes this as the way to navigate in Spotlight. |
| MC.04.05.b023 | Press an arrow key. | Move to another app. | While navigating in the apps window. |
| MC.04.05.b024 | Click a category along the top of the apps window, such as Productivity & Finance. | Filter apps by category. | — |
| MC.04.05.b025 | Click the omitted control shown in the source, then choose Grid or List. | Change the apps window view. | The source does not identify the control by name. |
| MC.04.05.b026 | Choose the option to show iPhone apps. | Show iPhone apps in the apps window. | Presented among the view choices. |
| MC.04.05.b027 | Download a game from Apple Games. | The game is added to the Arcade category in the apps window of Spotlight. | — |
| MC.04.05.b028 | Use a compatible game controller. | Access and navigate Arcade in the apps window. | A compatible game controller is being used. |
| MC.04.05.b029 | Download an app from the App Store. | The app automatically appears in Spotlight. | — |
| MC.04.05.b030 | Drag an app to the Applications folder in Finder. | The app is shown in Spotlight. | The app was not downloaded from the App Store. |
| MC.04.05.b031 | Drag an app from the apps browsing window to the Dock. | The app is added to the Dock and remains in Spotlight. | — |
| MC.04.05.b032 | Drag an app to the desktop. | Place the app on the desktop for easy access. | — |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.04.05.k004 | Return | Submit the app name entered in the search field. | Searching for apps in Spotlight. |
| MC.04.05.k005 | Esc | Close Spotlight without opening an app. | Spotlight is open. |
| MC.04.05.k006 | ⌘-1 | Enter the apps window for navigation. | The page says to click Dock, or click the menu bar and press this shortcut. |
| MC.04.05.k007 | Arrow key | Move to another app. | Navigating in the apps window. |

System Settings paths referenced: Screen Time settings

#### Independent reports (MC.04.05)

| ID | Kind | Area | Claim | Source |
|---|---|---|---|---|
| MC.04.05.x01 | new | tiling | macOS Sequoia introduced native window tiling, including snapping a window by dragging it to a screen edge and opening a layout picker by hovering over the green traffic-light button. | T4 [Appish](https://appish.app/blog/mac-window-tiling-shortcuts-not-working-fix) |
| MC.04.05.x02 | behavior | tiling | The setting “Tile by dragging windows to screen edges” enables edge-drag tiling; if tiling is off, shortcuts will not work. | T4 [Appish](https://appish.app/blog/mac-window-tiling-shortcuts-not-working-fix) |
| MC.04.05.x03 | behavior | tiling | “Hold Option key while dragging to tile” is a separate setting and is off by default. | T4 [Appish](https://appish.app/blog/mac-window-tiling-shortcuts-not-working-fix) |
| MC.04.05.x04 | bug | tiling | The article reports that macOS point updates, including 15.3 and 15.4, have reset tiling settings for some users. | T4 [Appish](https://appish.app/blog/mac-window-tiling-shortcuts-not-working-fix) |
| MC.04.05.x05 | bug | tiling | The layout picker may be absent in Electron apps or apps that draw their own window controls, after switching displays, or after waking from sleep. | T4 [Appish](https://appish.app/blog/mac-window-tiling-shortcuts-not-working-fix) |
| MC.04.05.x06 | workaround | tiling | To bring up the layout picker, the article recommends hovering rather than clicking and pausing deliberately; holding Option while hovering may reveal additional options on some Sequoia versions. | T4 [Appish](https://appish.app/blog/mac-window-tiling-shortcuts-not-working-fix) |
| MC.04.05.x07 | workaround | tiling | If dragging to a screen edge does not snap a window, the article recommends checking “Tile by dragging windows to screen edges” and dragging by the title bar rather than a toolbar or sidebar. | T4 [Appish](https://appish.app/blog/mac-window-tiling-shortcuts-not-working-fix) |
| MC.04.05.x08 | behavior | tiling | The article describes Sequoia's native tiling as supporting left half, right half, quarters, and full-screen layouts. | T4 [Appish](https://appish.app/blog/mac-window-tiling-shortcuts-not-working-fix) |
| MC.04.05.x09 | new | other | macOS 27 adds more high-resolution and high-refresh-rate display modes. The article recommends checking “System Settings” → “Displays” after updating. | T4 [Lets Suite](https://www.lets-suite.com/blog/macos-27-window-management) |
| MC.04.05.x10 | behavior | tiling | Native tiling supports halves, quarters, and corners; it does not add thirds, two-thirds, or custom grids, leaving the same limits as Tahoe. | T4 [Lets Suite](https://www.lets-suite.com/blog/macos-27-window-management) |
| MC.04.05.x11 | behavior | tiling | For placement beyond a half, the article says users must use the mouse: drag a window to an edge or hover over the green button. | T4 [Lets Suite](https://www.lets-suite.com/blog/macos-27-window-management) |
| MC.04.05.x12 | workaround | tiling | The article says Let's Tile can show a grid over the current window, with placement controlled by arrow keys and confirmed with Return. | T4 [Lets Suite](https://www.lets-suite.com/blog/macos-27-window-management) |
| MC.04.05.x13 | workaround | tiling | The article says Let's Tile's free tier offers a 3 × 2 grid, including halves, thirds, two-thirds, quarters, and full screen; Pro adds custom grids, per-display grids, and moving a window to another monitor while tiling. | T4 [Lets Suite](https://www.lets-suite.com/blog/macos-27-window-management) |
| MC.04.05.x14 | behavior | tiling | The article says native window-tiling shortcuts are Globe-Control-arrow combinations and tile windows to halves or quarters, but not thirds. | T4 [Lets Suite](https://www.lets-suite.com/blog/full-keyboard-access-mac) |
| MC.04.05.x15 | bug | tiling | Fixed an issue where an NSSegmentedCell could render at the wrong location under the Liquid Glass appearance. | T4 [BetaProfile](https://betaprofile.dev/macos-27-rc/) |

### MC.04.06 Sheets & panels

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).

Also covered by: [Move and arrange app windows on Mac](https://support.apple.com/guide/mac-help/mchlp2469/27/mac/27), [Tile windows on Mac](https://support.apple.com/guide/mac-help/mchlef287e5d/27/mac/27)


#### Sheets

T1 · HIG (developer guidance) · [Apple HIG](https://developer.apple.com/design/human-interface-guidelines/sheets) · macOS 27 text present: no

The guidance describes macOS sheets as modal, cardlike views attached to a parent window. It recommends using them for scoped tasks, choosing an appropriate size, and using windows or panels when they better fit the workflow.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.04.06.e001 | sheet | view | In front of its parent window | A rounded, cardlike modal view. The parent window is dimmed while the sheet is onscreen. |
| MC.04.06.e002 | parent window | window | Behind the sheet | The window to which the sheet is attached. People can’t interact with it until they dismiss the sheet. |
| MC.04.06.e003 | Cancel | button | In a sheet; placement varies by platform | Dismisses the sheet without saving changes. |
| MC.04.06.e004 | Close | button | In a sheet; placement varies by platform | An alternative label for a button that dismisses a sheet without saving changes. |
| MC.04.06.e005 | Done | button | In a sheet; placement varies by platform | Dismisses the sheet after completing a task or explicitly saving changes. |
| MC.04.06.e006 | Back | button | In a multi-step sheet flow | Navigates to a previous step or parent view; it isn’t intended to dismiss the sheet. |
| MC.04.06.e007 | panel | panel | In the app, alongside its windows | An alternative to a sheet when people need to repeatedly provide input and observe results. |
| MC.04.06.e008 | modeless document-related panels | panel | Associated with a document window | When a sheet opens for a document window, these panels are brought forward with the parent window. |

| ID | Context | Guideline | Conditions |
|---|---|---|---|
| MC.04.06.b001 | A sheet is presented in macOS. | It is modal and prevents interaction with its parent window until dismissed. It appears as a rounded cardlike view over the parent, which is dimmed. | People are still expected to interact with other app windows before dismissing the sheet. |
| MC.04.06.b002 | A sheet opens. | Bring its parent window to the front; if the parent is a document window, also bring its modeless document-related panels forward. | macOS guidance. |
| MC.04.06.b003 | People want to interact with another window in the same app while a sheet is open. | Allow them to bring that window forward without first dismissing the sheet. | The sheet remains open. |
| MC.04.06.b004 | Choosing a sheet’s default size. | Use a size appropriate for its content. Consider supporting resizing when people may need to expand the contents for a clearer view. | People don’t generally expect to resize macOS sheets. |
| MC.04.06.b005 | A task is complex or prolonged, such as editing a document or viewing media. | Consider using a new window or full-screen mode instead of a sheet. | For document editing, a separate window can suit the self-contained task; full-screen mode can help with viewing media. |
| MC.04.06.b006 | People need supplementary items that affect the main task while they continue interacting with the main window. | Consider using a panel instead of a sheet. | macOS guidance. |
| MC.04.06.b007 | People need to provide input repeatedly and observe results, such as checking individual find-and-replace operations. | Use a panel instead of a sheet. | macOS guidance. |
| MC.04.06.b008 | A sheet is closed. | People expect to return to the parent view or window. | General sheet guidance. |
| MC.04.06.b009 | A task within a sheet would cause another sheet to appear. | Close the first sheet before displaying the next one; if necessary, show the first sheet again after the second is dismissed. | Display only one sheet at a time from the main interface. |
| MC.04.06.b010 | A sheet has a Done button. | Pair it with a Cancel button or a Back button, and avoid showing Cancel, Done, and Back together. | General sheet guidance. |

API names: `present`

#### Panels

T1 · HIG (developer guidance) · [Apple HIG](https://developer.apple.com/design/human-interface-guidelines/panels) · macOS 27 text present: no

A macOS panel typically floats above other open windows and provides supplementary controls, options, or information related to the active window or current selection. The guidance covers panel content, titles, visibility, Window menu treatment, and HUD-style panels.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.04.06.e009 | panel | window | Throughout | A typically floating, less prominent window that provides supplementary controls, options, or information related to the active window or current selection. |
| MC.04.06.e010 | HUD | window | HUD-style panels | A panel with a darker, translucent appearance, suited to some highly visual or immersive experiences. |
| MC.04.06.e011 | inspector | view | Best practices | Displays details of the currently selected item and automatically updates when the item or selection changes. |
| MC.04.06.e012 | Info window | window | Best practices | A regular window for information that stays the same when the selected item changes. |
| MC.04.06.e013 | split view pane | view | Best practices | An alternative layout that may be used to present an inspector. |
| MC.04.06.e014 | slider | control | Best practices | An example of a direct adjustment control recommended for panels. |
| MC.04.06.e015 | stepper | control | Best practices | An example of a direct adjustment control recommended for panels. |
| MC.04.06.e016 | title bar | control | Best practices | Lets people position a panel where they want. |
| MC.04.06.e017 | Window menu | menu | Best practices | May contain commands to show or hide panels, but panels should not appear in its documents list. |
| MC.04.06.e018 | minimize button | button | Best practices | The guidance generally recommends not making this button available on a panel. |
| MC.04.06.e019 | disclosure triangle | control | HUD-style panels | Identified as an exception among system-provided controls that match a HUD's appearance. |
| MC.04.06.e020 | Fonts | panel | Best practices | A familiar macOS panel title used as an example. |
| MC.04.06.e021 | Colors | panel | Best practices | A familiar macOS panel title used as an example. |
| MC.04.06.e022 | Inspector | panel | Best practices | An example panel title; it is also used as an example of a title that can supply enough context on its own. |
| MC.04.06.e023 | Fonts window | window | Best practices | An example of how to refer to a panel in help documentation. |
| MC.04.06.e024 | Colors window | window | Best practices | An example of how to refer to a panel in help documentation. |
| MC.04.06.e025 | QuickTime Player | other | HUD-style panels | Given as an example of an app that uses a HUD to display inspector information. |

| ID | Context | Guideline | Conditions |
|---|---|---|---|
| MC.04.06.b011 | When supplementary content is relevant to the current task or selection on platforms other than macOS | Consider using a modal view. | The page says panels are not supported on iOS, iPadOS, tvOS, visionOS, or watchOS. |
| MC.04.06.b012 | When people need quick access to important controls or information related to content they are working with | Use a panel. | — |
| MC.04.06.b013 | When presenting inspector functionality | Consider using a panel; an inspector should update its details when the selected item changes. Use a regular window instead for an Info window whose contents stay the same, or consider a split view pane for an inspector. | — |
| MC.04.06.b014 | When choosing controls for a panel | Prefer simple adjustment controls such as sliders and steppers; avoid controls that require typing text or selecting items when possible. | Typing and selecting can require multiple steps. |
| MC.04.06.b015 | When naming a panel | Give it a brief title that describes its purpose, using a noun or noun phrase with title-style capitalization. | A panel needs a title bar so people can position it. |
| MC.04.06.b016 | When the app becomes active | Bring all of its open panels to the front, regardless of which window was active when a panel opened. | — |
| MC.04.06.b017 | When the app becomes inactive | Hide all of its panels. | — |
| MC.04.06.b018 | When listing documents in the Window menu | Do not include panels in the documents list; commands for showing or hiding panels may be included. | — |
| MC.04.06.b019 | When deciding whether a panel has a minimize button | In general, avoid making the minimize button available. | Panels display only when needed and disappear when the app is inactive. |
| MC.04.06.b020 | When referring to a panel in menus | Use the panel's title without the term “panel,” as in “Show Fonts,” “Show Colors,” or “Show Inspector.” | — |
| MC.04.06.b021 | When referring to a panel in help documentation | Refer to it by its title or, when clearer, append “window” to the title. | — |
| MC.04.06.b022 | When deciding whether to use a HUD | Prefer a standard panel. Use a HUD only in a media-oriented app presenting movies, photos, or slides; when a standard panel would obscure essential content; or when controls are not needed, except for the disclosure triangle. | Most system-provided controls do not match a HUD's appearance. |
| MC.04.06.b023 | When an app switches between modes, such as entering or leaving full-screen mode | Maintain one panel style; for example, retain the HUD style when leaving full-screen mode if the app uses a HUD in full-screen. | — |
| MC.04.06.b024 | When adding color to HUDs | Use color sparingly; small amounts of high-contrast color may highlight important information. | Too much color in the dark appearance of a HUD can be distracting. |
| MC.04.06.b025 | When sizing a HUD | Keep it small and unobtrusive; do not let it obscure the content it adjusts or compete for attention. | — |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.04.06.k001 | Up Arrow, Down Arrow, Left Arrow, Right Arrow | Navigate the symbols. | Documentation page navigation instructions. |

API names: `NSPanel`, `hudWindow`

### MC.04.07 Window restoration/persistence

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).

Also covered by: [Quit apps on Mac - Apple Support](https://support.apple.com/guide/mac-help/mchl834d18c2/27/mac/27)


#### Prevent apps and windows from reopening on Mac

T1 · OS documentation · [Apple Support](https://support.apple.com/en-us/102318) · macOS 27 text present: no

Explains how to prevent apps from reopening at login and how to prevent apps from restoring windows when opened, plus alternative steps and app-specific checks.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.04.07.e001 | Apple menu  | menu | Menu bar | Menu used to access System Settings or System Preferences, and to choose Log Out, Shut Down, or Restart. |
| MC.04.07.e002 | Reopen windows when logging back in | checkbox | Confirmation dialog shown when shutting down, restarting, or logging out | Checkbox controlling whether windows reopen at the next login. |
| MC.04.07.e003 | Log Out | menu item | Apple menu  | Can be chosen with the Option key held to bypass the confirmation dialog. |
| MC.04.07.e004 | Shut Down | menu item | Apple menu  | Can be chosen with the Option key held to bypass the confirmation dialog. |
| MC.04.07.e005 | Restart | menu item | Apple menu  | Can be chosen with the Option key held to bypass the confirmation dialog. |
| MC.04.07.e006 | System Settings | menu item | Apple menu  | Opens System Settings. |
| MC.04.07.e007 | Desktop & Dock | setting | System Settings sidebar | Settings section containing the option to close windows when quitting an application. |
| MC.04.07.e008 | Close windows when quitting an application | setting | System Settings > Desktop & Dock; scroll down on the right | When turned on, prevents apps from restoring windows they had open when last quit. |
| MC.04.07.e009 | System Preferences | menu item | Apple menu  | Opens System Preferences in earlier versions of macOS. |
| MC.04.07.e010 | General | setting | System Preferences | Contains the option to close windows when quitting an app in earlier versions of macOS. |
| MC.04.07.e011 | Close windows when quitting an app | setting | System Preferences > General | Select this option to prevent apps from restoring windows they had open when last quit. |
| MC.04.07.e012 | Shift | other | Keyboard | Press and hold while opening an app to make it forget the windows that were open the last time it was quit. |
| MC.04.07.e013 | Safe mode | other | Mac startup | Starting up in safe mode and then restarting normally makes the Mac forget apps and windows that were open at the last logout or restart. |
| MC.04.07.e014 | Login items | other | Mac login behavior | Items that open every time the user logs in to the Mac; apps can be added to or removed from login items. |
| MC.04.07.e015 | Safari | other | App settings example | The article gives Safari as an example of an app with a setting controlling which windows it opens with. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.04.07.b001 | Log in to a Mac or open an app | Apps and their windows might automatically reopen. | General behavior described by the article. |
| MC.04.07.b002 | Shut down, restart, or log out of a Mac | A confirmation dialog should appear asking whether the user is sure. | When the confirmation dialog is shown. |
| MC.04.07.b003 | Deselect “Reopen windows when logging back in” in the confirmation dialog | Windows will not reopen when logging back in. | The checkbox is available in the shutdown, restart, or logout confirmation dialog. |
| MC.04.07.b004 | The Mac is forced to turn off or restart immediately, so the confirmation dialog is not shown | The Mac uses the most recent “Reopen windows when logging back in” setting when it starts up again. | The confirmation dialog was not shown because shutdown or restart was forced. |
| MC.04.07.b005 | Press and hold Option while choosing Log Out, Shut Down, or Restart from the Apple menu  | The confirmation dialog is bypassed. | The most recent “Reopen windows when logging back in” setting is used at the next startup. |
| MC.04.07.b006 | Turn on “Close windows when quitting an application” | Apps do not restore the windows they had open when last quit. | macOS Ventura or later; the option is in System Settings > Desktop & Dock. |
| MC.04.07.b007 | Select “Close windows when quitting an app” | Apps do not restore the windows they had open when last quit. | Earlier versions of macOS; the option is in System Preferences > General. |
| MC.04.07.b008 | Press and hold Shift while opening an app | The app forgets any windows that were open the last time it was quit. | This does not change any settings. |
| MC.04.07.b009 | Start up in safe mode, then restart normally | The Mac forgets any apps and windows that were open the last time the user logged out or restarted. | This does not change any settings. |
| MC.04.07.b010 | Change an app's own settings | The app may stop reopening at login or stop reopening its windows when opened. | Some apps provide settings for these behaviors; Safari is given as an example of an app with a setting controlling which windows it opens with. |
| MC.04.07.b011 | An app is a login item | The app opens every time the user logs in to the Mac. | Login items can be added or removed. |
| MC.04.07.b012 | An app keeps opening at login but is not a login item | It might have installed other software that causes this behavior. | The article recommends checking the app’s documentation or contacting the app developer. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.04.07.k001 | Option | Press and hold while choosing Log Out, Shut Down, or Restart to bypass the confirmation dialog. | Apple menu  |
| MC.04.07.k002 | Shift | Press and hold while opening an app to make it forget the windows that were open the last time it was quit. | Opening an app |

System Settings paths referenced: System Settings > Desktop & Dock > Close windows when quitting an application; System Preferences > General > Close windows when quitting an app

#### Shut down or restart your Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp2522/27/mac/27) · macOS 27 text present: no

Instructions for shutting down or restarting a Mac, including guidance about reopening app windows, laptop displays, peripherals, and forced shutdown.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.04.07.e016 | Apple menu | menu | On the Mac | Menu used to access the shutdown and restart commands. |
| MC.04.07.e017 | Reopen windows when logging back in | setting | In the shutdown or restart flow | Option to control whether open app windows reopen after the Mac starts or restarts. |
| MC.04.07.e018 | Shut Down | menu item | Apple menu | Command to shut down the Mac. |
| MC.04.07.e019 | Restart | menu item | Apple menu | Command to restart the Mac. |
| MC.04.07.e020 | power button | control | On the Mac | Press and hold to force a shutdown if necessary. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.04.07.b013 | Choose Apple menu, optionally deselect “Reopen windows when logging back in,” then click “Shut Down.” | The Mac shuts down. | The option can be deselected if you do not want open app windows to reopen when you start the Mac again. |
| MC.04.07.b014 | The Mac's screen is black. | The Mac is completely shut down. | Some computers also have no active power light and no fan or drive noise. |
| MC.04.07.b015 | Close a Mac laptop's display while it is shutting down. | The Mac may not shut down properly and may start up more slowly next time. | Wait for the laptop to shut down completely before closing the display. |
| MC.04.07.b016 | Open the case to install additional memory or other devices inside the computer. | Shut down the Mac first. | Always do this before opening the case for this purpose. |
| MC.04.07.b017 | Connect or unplug certain peripheral devices. | You may need to shut down the Mac first. | If unsure, refer to the documentation that came with the device. |
| MC.04.07.b018 | Connect or disconnect USB-C, Thunderbolt, or Ethernet cables. | You do not need to shut down the Mac. | Applies to the listed cables. |
| MC.04.07.b019 | Press and hold the power button until the Mac shuts down. | The Mac is forced to shut down. | Use if necessary, for example, if the Mac becomes unresponsive. Unsaved changes in open documents may be lost. |
| MC.04.07.b020 | Choose Apple menu, optionally deselect “Reopen windows when logging back in,” then click “Restart.” | The Mac restarts. | The option can be deselected if you do not want open app windows to reopen when the Mac restarts. |

## 14.5 MC.05 Spaces / Mission Control / Stage Manager


### MC.05.01 Spaces

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).


#### Work in multiple spaces on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mh14112/27/mac/27) · macOS 27 text present: no

Mission Control lets you create and switch among additional desktops called spaces, move windows between spaces, assign apps to spaces, and delete spaces.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.05.01.e001 | Mission Control | view | On Mac | The view used to create, navigate, and manage spaces. |
| MC.05.01.e002 | Spaces bar | view | At the top of the screen in Mission Control | Shows space thumbnails, including thumbnails for apps in full screen or Split View. |
| MC.05.01.e003 | space thumbnail | thumbnail | In the Spaces bar | Click a space thumbnail to open that space. |
| MC.05.01.e004 | space-creation control | button | In the Spaces bar | The article says to click this control to create a space; its icon or label is not shown in the provided text. |
| MC.05.01.e005 | app window | window | In a space | Windows in other spaces are not shown while working in a given space. |
| MC.05.01.e006 | Options | menu | In the Dock app icon's context menu | Contains assignment choices under Assign To. |
| MC.05.01.e007 | Assign To | menu | Options submenu for a Dock app icon | Contains choices for assigning an app to desktops or displays. |
| MC.05.01.e008 | All Desktops | menu item | Options > Assign To | Makes the app open in every space. |
| MC.05.01.e009 | This Desktop | menu item | Options > Assign To | Makes the app open only in the current space; if used full screen, it appears in its own space. |
| MC.05.01.e010 | Desktop on Display [number] | menu item | Options > Assign To; when more than one display is available | Makes the app open in the current space on a specific display. |
| MC.05.01.e011 | None | menu item | Options > Assign To | Makes the app open in whichever space is being used at the time. |
| MC.05.01.e012 | Dock | view | On Mac | The app icon can be Control-clicked to access Options. |
| MC.05.01.e013 | Apple menu | menu | Menu bar | The article refers to using it to open Desktop & Dock settings. |
| MC.05.01.e014 | Desktop & Dock | panel | System Settings | Settings panel referenced for changing the behavior when switching to an app. |
| MC.05.01.e015 | Open Desktop & Dock settings for me | other | Link in the article | Opens Desktop & Dock settings. |
| MC.05.01.e016 | Exit button | button | On a full screen or Split View thumbnail in the Spaces bar | Click to stop using an app in full screen or Split View. |
| MC.05.01.e017 | full screen | view | Shown as a thumbnail in the Spaces bar | A full screen app has its own thumbnail in the Spaces bar. |
| MC.05.01.e018 | Split View | view | Shown as a thumbnail in the Spaces bar | A Split View app arrangement has a thumbnail in the Spaces bar. |
| MC.05.01.e019 | System Settings | app | On Mac | The article says to assign System Settings to all desktops before changing wallpaper in each space. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.05.01.b001 | Enter Mission Control. | The Spaces bar can be used to create and manage spaces. | If working with apps in full screen or Split View, their thumbnails are shown in the Spaces bar. |
| MC.05.01.b002 | Click the space-creation control in the Spaces bar. | A new space is created. | Up to 16 spaces can be created. |
| MC.05.01.b003 | Click a space thumbnail. | That space opens. | The thumbnail is in the Spaces bar. |
| MC.05.01.b004 | Assign System Settings to all desktops, then change the wallpaper in each space. | Different desktop pictures can be used for different spaces. | Tip in the article. |
| MC.05.01.b005 | Work in a space. | Only the windows in that space are shown. | When using spaces. |
| MC.05.01.b006 | Swipe left or right with three or four fingers. | Move between spaces. | On a trackpad. |
| MC.05.01.b007 | Swipe with two fingers. | Move between spaces. | On a Magic Mouse. |
| MC.05.01.b008 | Press Control and the Right or Left arrow key. | Move between spaces. | On Mac. |
| MC.05.01.b009 | Enter Mission Control, move the pointer to the top edge of the screen, then click a space in the Spaces bar. | The selected space opens. | On Mac. |
| MC.05.01.b010 | Drag an app window to the edge of the screen and pause. | The window moves to the next space. | After a moment. |
| MC.05.01.b011 | Go to the space containing the window, enter Mission Control, then drag the window to the desired space. | The window moves to the selected space. | On Mac. |
| MC.05.01.b012 | Drag a window onto a full screen app in the Spaces bar. | The apps can be used in Split View. | While moving a window between spaces in Mission Control. |
| MC.05.01.b013 | Control-click an app's icon in the Dock, choose Options, then choose an option under Assign To. | The app is assigned according to the selected option. | When using two or more spaces; the app may need to be opened first to show its Dock icon. |
| MC.05.01.b014 | Choose All Desktops under Assign To. | The app opens in every space. | For an app assigned through Dock > Options. |
| MC.05.01.b015 | Choose This Desktop under Assign To. | The app opens only in the current space. | If the app is used full screen, it appears in its own space. |
| MC.05.01.b016 | Choose Desktop on Display [number] under Assign To. | The app opens in the current space on a specific display. | Available when more than one display is available. |
| MC.05.01.b017 | Choose None under Assign To. | The app opens in whichever space is being used at the time. | For an app assigned through Dock > Options. |
| MC.05.01.b018 | Switch to an app. | By default, the desktop automatically switches to a space that has open windows for the app. | The article's example says a new TextEdit document created in Desktop 3 opens in Desktop 2 if TextEdit windows are already open in Desktop 2. |
| MC.05.01.b019 | Change the setting for automatic switching to a space with open app windows. | The default app-switching behavior can be changed. | The article directs readers to Apple menu > System Settings > Desktop & Dock. |
| MC.05.01.b020 | Enter Mission Control, move the pointer to the top edge of the screen, hold the pointer over a space in the Spaces bar, then click its delete control. | The space is deleted. | If the space contains open windows, they are moved to another space; the delete control's icon or label is not shown in the provided text. |
| MC.05.01.b021 | Move the pointer over a full screen or Split View thumbnail in the Spaces bar, then click Exit button. | Stop using the app in full screen or Split View. | The thumbnail is in the Spaces bar. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.05.01.k001 | Three- or four-finger swipe left or right | Move between spaces. | Trackpad. |
| MC.05.01.k002 | Two-finger swipe | Move between spaces. | Magic Mouse. |
| MC.05.01.k003 | Control-Left Arrow or Control-Right Arrow | Move between spaces. | On Mac. |

System Settings paths referenced: Apple menu > System Settings > Desktop & Dock; System Settings assigned to All Desktops (for changing the wallpaper in each space)

#### Independent reports (MC.05.01)

| ID | Kind | Area | Claim | Source |
|---|---|---|---|---|
| MC.05.01.x01 | changed | Spaces | Apple calls out faster animation and interaction performance in Spaces; the article says what Spaces does does not change. | T4 [Lets Suite](https://www.lets-suite.com/blog/macos-27-window-management) |
| MC.05.01.x02 | changed | Spaces | macOS does a better job restoring window positions, Spaces, and screen assignments when a display is disconnected and reconnected. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MC.05.01.x03 | changed | Spaces | The post reports smoother animations in Mission Control and Spaces. | T5 [MacRumors Forums](https://forums.macrumors.com/threads/macos-27-all-the-little-things.2483520/) |
| MC.05.01.x04 | changed | Spaces | Animations in Spaces have been improved. | T4 [9to5Mac](https://9to5mac.com/2026/09/14/macos-27-golden-gate-now-available-here-is-everything-new/) |

### MC.05.02 Mission Control

Status: **PARTIAL**. Guide captured; third-party reports of changed Dock accessibility notifications and animation bugs (GAP-B5-06).


#### View open windows and spaces in Mission Control on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mh35798/27/mac/27) · macOS 27 text present: no

Mission Control displays open windows on the desktop in a single layer. Full-screen apps, Split View, and created desktop spaces appear as thumbnails in the Spaces bar.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.05.02.e001 | Mission Control | view | Mac desktop | Shows open windows in a single layer. |
| MC.05.02.e002 | Spaces bar | view | Along the top edge of the screen in Mission Control | Shows thumbnails for apps in full screen or Split View, and desktop spaces that were created. |
| MC.05.02.e003 | Split View | view | Spaces bar and desktop | Full-screen apps or apps in Split View are shown as thumbnails in the Spaces bar. |
| MC.05.02.e004 | Mission Control key | other | Keyboard | Used to enter or leave Mission Control and in the actions described for copying items between windows. |
| MC.05.02.e005 | Trackpad | setting | Trackpad settings | Settings for changing or turning off the described trackpad gestures. |
| MC.05.02.e006 | Keyboard | setting | Keyboard settings | Settings for the Mission Control key and for changing keyboard shortcuts. |
| MC.05.02.e007 | Magic Mouse | other | Mac | A two-finger double-tap can enter or leave Mission Control when the gesture is turned on. |
| MC.05.02.e008 | Mouse | setting | Mouse settings | Settings where the Magic Mouse gesture can be turned on or off. |
| MC.05.02.e009 | Desktop & Dock | setting | Desktop & Dock settings | Contains settings for the window-drag feature and additional Mission Control options. |
| MC.05.02.e010 | App Exposé | setting | Trackpad settings | When enabled, a three- or four-finger swipe down can show all open windows for the current app. |
| MC.05.02.e011 | Group windows by application | setting | Desktop & Dock settings, below Mission Control | When turned on, Control-Down Arrow shows open windows grouped by app. |
| MC.05.02.e012 | Apple menu | menu | Mac menu bar | The article directs the reader to choose this menu to open Desktop & Dock settings. |
| MC.05.02.e013 | Open Desktop & Dock settings for me | button | Below the article text | Link that opens Desktop & Dock settings. |
| MC.05.02.e014 | See also | other | Below the article | Section containing related support links. |
| MC.05.02.e015 | Work in multiple spaces on Mac | other | See also | Related support link. |
| MC.05.02.e016 | Use your iPad as a second display for your Mac | other | See also | Related support link. |
| MC.05.02.e017 | View and customize mouse or trackpad gestures on Mac | other | See also | Related support link. |
| MC.05.02.e018 | Move and arrange app windows on Mac | other | See also | Related support link. |
| MC.05.02.e019 | Tile windows on Mac | other | See also | Related support link. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.05.02.b001 | Swipe up using three fingers on a trackpad | Enter Mission Control. | Trackpad. |
| MC.05.02.b002 | Swipe down using three fingers on a trackpad | Leave Mission Control. | Trackpad. |
| MC.05.02.b003 | Press the Mission Control key | Enter or leave Mission Control. | Keyboard. |
| MC.05.02.b004 | Double-tap using two fingers | Enter or leave Mission Control. | Magic Mouse gesture is turned on. |
| MC.05.02.b005 | Drag a window up to the top of the screen | Enter Mission Control. | The feature is turned on in Desktop & Dock settings. |
| MC.05.02.b006 | Click the desktop | Leave Mission Control. | After entering Mission Control by dragging a window to the top of the screen. |
| MC.05.02.b007 | Enter Mission Control on a second display | Only the open windows and spaces being used on that display are shown. | A second display is in use. |
| MC.05.02.b008 | Press Control-Down Arrow | Show all open windows for the current app. | Mission Control keyboard shortcut. |
| MC.05.02.b009 | Swipe down with three or four fingers | Show all open windows for the current app. | App Exposé is enabled in Trackpad settings. |
| MC.05.02.b010 | Press Control-Down Arrow again, or swipe up | Return to the desktop. | After showing all open windows for the current app. |
| MC.05.02.b011 | Turn on Group windows by application, then press Control-Down Arrow | Show all open windows grouped by app. | The setting is in Desktop & Dock settings below Mission Control. |
| MC.05.02.b012 | Press Command-Mission Control, or spread the thumb and three fingers apart on the trackpad | Move open windows aside to show the desktop. | The trackpad gesture uses the thumb and three fingers. |
| MC.05.02.b013 | Press Command-Mission Control again, or pinch the thumb and fingers together on the trackpad | Move the windows back. | After moving the windows aside. |
| MC.05.02.b014 | Copy text or an image, then press the Mission Control key | Use Mission Control to copy the item between windows. | Text or an image has been copied. |
| MC.05.02.b015 | Copy a file or folder, then press Command-Mission Control | Copy the file or folder to the desktop. | A file or folder has been copied. |
| MC.05.02.b016 | Start dragging a file or folder from a window, press Command-Mission Control, then drop the item | Move the file or folder to the desktop; the shortcut moves all windows aside to show the desktop. | The item is being dragged from a window. |
| MC.05.02.b017 | Press Command-Mission Control again | Move the windows back. | After moving a dragged item to the desktop. |
| MC.05.02.b018 | Drag a window from Mission Control onto the thumbnail of a full-screen app in the Spaces bar, then click the thumbnail | Use two apps in Split View. | The target thumbnail is for a full-screen app. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.05.02.k001 | Three-finger swipe up | Enter Mission Control. | Trackpad. |
| MC.05.02.k002 | Three-finger swipe down | Leave Mission Control. | Trackpad. |
| MC.05.02.k003 | Mission Control key | Enter or leave Mission Control. | Keyboard. |
| MC.05.02.k004 | Two-finger double-tap | Enter or leave Mission Control. | Magic Mouse; gesture must be turned on. |
| MC.05.02.k005 | Control-Down Arrow | Show all open windows for the current app; press again to return to the desktop. | Mission Control. |
| MC.05.02.k006 | Three- or four-finger swipe down | Show all open windows for the current app. | Trackpad; App Exposé must be enabled. |
| MC.05.02.k007 | Swipe up | Return to the desktop from the current app's open-window view. | Trackpad, after using the App Exposé gesture. |
| MC.05.02.k008 | Command-Mission Control | Move open windows aside to show the desktop; press again to move them back. | Keyboard shortcut. |
| MC.05.02.k009 | Spread thumb and three fingers apart | Move open windows aside to show the desktop. | Trackpad. |
| MC.05.02.k010 | Pinch thumb and fingers together | Move the windows back. | Trackpad, after moving windows aside. |
| MC.05.02.k011 | Mission Control key | After copying text or an image, use Mission Control to copy it between windows. | Copying between windows. |
| MC.05.02.k012 | Command-Mission Control | After copying a file or folder, copy it to the desktop; while dragging a file or folder, move windows aside to show the desktop. | Copying or moving files and folders to the desktop. |

System Settings paths referenced: System Settings > Trackpad; System Settings > Keyboard; System Settings > Mouse; System Settings > Desktop & Dock; System Settings > Desktop & Dock > Mission Control > Group windows by application

#### Independent reports (MC.05.02)

| ID | Kind | Area | Claim | Source |
|---|---|---|---|---|
| MC.05.02.x01 | behavior | Mission Control | The author reports that selecting a different window in Mission Control feels difficult, like rearranging apps on an iPhone home screen or repositioning an image in Word. | T5 [Reddit r/MacOS](https://www.reddit.com/r/MacOS/comments/1wlax16/mission_control_expose_is_broken_in_macos_27/) |
| MC.05.02.x02 | bug | Mission Control | The author describes Mission Control/Exposé in macOS 27 as glitchy and slow, with bugs and stuttering animations. (translated) | T5 [Reddit r/MacOS](https://www.reddit.com/r/MacOS/comments/1wlax16/mission_control_expose_is_broken_in_macos_27/) |
| MC.05.02.x03 | workaround | Mission Control | A commenter reports that macOS 27.0.1 made Mission Control animations smoother and stopped windows moved to another Space from ending up in unexpected locations. | T5 [Reddit r/MacOS](https://www.reddit.com/r/MacOS/comments/1wlax16/mission_control_expose_is_broken_in_macos_27/) |
| MC.05.02.x04 | bug | Mission Control | A commenter reports that after swiping up to open Mission Control, selecting an app's icon in the Dock causes a stuttering transition and does not bring the selected app forward. | T5 [Reddit r/MacOS](https://www.reddit.com/r/MacOS/comments/1wlax16/mission_control_expose_is_broken_in_macos_27/) |
| MC.05.02.x05 | bug | Mission Control | On macOS 27, a small off-screen window kept by macOS could be mistaken for the Mission Control overlay, causing AltTab to refuse to open its switcher. The detector now treats a shield as an overlay only if it is at least as large as a screen. | T2 [GitHub lwouis/alt-tab-macos](https://github.com/lwouis/alt-tab-macos/commit/cbd8e912dded9a7f137647c1f496bfe06ed875df) |
| MC.05.02.x06 | behavior | Mission Control | The commit says the Dock no longer posts the accessibility notifications used to detect Mission Control and App Exposé on macOS 27; the app therefore identifies these gestures from WindowManager surfaces. (translated) | T2 [GitHub lwouis/alt-tab-macos](https://github.com/lwouis/alt-tab-macos/commit/cbd8e912dded9a7f137647c1f496bfe06ed875df) |
| MC.05.02.x07 | behavior | Mission Control | A screen-sized WindowManager shield indicates App Exposé; a shield together with the Spaces bar indicates Mission Control. (translated) | T2 [GitHub lwouis/alt-tab-macos](https://github.com/lwouis/alt-tab-macos/commit/cbd8e912dded9a7f137647c1f496bfe06ed875df) |
| MC.05.02.x08 | behavior | Mission Control | Without a shield, a WindowManager Show Desktop overlay indicates Show Desktop. | T2 [GitHub lwouis/alt-tab-macos](https://github.com/lwouis/alt-tab-macos/commit/cbd8e912dded9a7f137647c1f496bfe06ed875df) |
| MC.05.02.x09 | behavior | Mission Control | The overlay detector reads the on-screen window list using `CGWindowListCopyWindowInfo` and obtains active display sizes using Core Graphics APIs. | T2 [GitHub lwouis/alt-tab-macos](https://github.com/lwouis/alt-tab-macos/commit/cbd8e912dded9a7f137647c1f496bfe06ed875df) |
| MC.05.02.x10 | behavior | Mission Control | Mission Control, opened with ⌃↑, shows everything; Spaces continue to work, and each Space has its own stage and strip. | T4 [Lets Suite](https://www.lets-suite.com/blog/how-to-use-stage-manager-mac) |
| MC.05.02.x11 | changed | Mission Control | Apple calls out faster animation and interaction performance in Mission Control; the article says its functionality does not change. | T4 [Lets Suite](https://www.lets-suite.com/blog/macos-27-window-management) |
| MC.05.02.x12 | changed | Mission Control | Mission Control and Spaces animations are smoother. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MC.05.02.x13 | changed | Mission Control | Animations in Mission Control have been improved. | T4 [9to5Mac](https://9to5mac.com/2026/09/14/macos-27-golden-gate-now-available-here-is-everything-new/) |

### MC.05.03 App Exposé

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).

Also covered by: [View open windows and spaces in Mission Control on Mac](https://support.apple.com/guide/mac-help/mh35798/27/mac/27)


#### Use Multi-Touch gestures on Mac

T1 · OS documentation · [Apple Support](https://support.apple.com/en-us/102482) · macOS 27 text present: no

Explains gestures for a Multi-Touch trackpad or Magic Mouse, including their actions, hardware requirements, and where to adjust gesture settings.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.05.03.e001 | Trackpad gestures | view | Main content heading | Section describing gestures for a Magic Trackpad or built-in Multi-Touch trackpad. |
| MC.05.03.e002 | Magic Mouse gestures | view | Main content heading | Section describing gestures for Magic Mouse. |
| MC.05.03.e003 | Tap to click | gesture | Trackpad gestures | Tap with one finger to click. |
| MC.05.03.e004 | Secondary click (right-click) | gesture | Trackpad gestures and Magic Mouse gestures | For the trackpad, click or tap with two fingers. For Magic Mouse, click the right side. |
| MC.05.03.e005 | Smart zoom | gesture | Trackpad gestures and Magic Mouse gestures | For the trackpad, double-tap with two fingers. For Magic Mouse, double-tap with one finger. |
| MC.05.03.e006 | Scroll | gesture | Trackpad gestures and Magic Mouse gestures | Slide two fingers on the trackpad or one finger on Magic Mouse. |
| MC.05.03.e007 | Zoom in or out | gesture | Trackpad gestures | Pinch with two fingers. |
| MC.05.03.e008 | Rotate | gesture | Trackpad gestures | Move two fingers around each other. |
| MC.05.03.e009 | Swipe between pages | gesture | Trackpad gestures and Magic Mouse gestures | Swipe left or right with two fingers on the trackpad or one finger on Magic Mouse. |
| MC.05.03.e010 | Open Notification Center | gesture | Trackpad gestures | Swipe left from the right edge with two fingers. |
| MC.05.03.e011 | Three finger drag | gesture | Trackpad gestures | Use three fingers to drag items on screen; click or tap to drop. Turn on in Accessibility settings. |
| MC.05.03.e012 | Look up and data detectors | gesture | Trackpad gestures | Tap with three fingers. |
| MC.05.03.e013 | Show desktop | gesture | Trackpad gestures | Spread your thumb and three fingers apart. |
| MC.05.03.e014 | Apps | gesture | Trackpad gestures | Pinch your thumb and three fingers together. |
| MC.05.03.e015 | Mission Control | gesture | Trackpad gestures and Magic Mouse gestures | Swipe up with four fingers on the trackpad, or double-tap with two fingers on Magic Mouse. |
| MC.05.03.e016 | App Exposé | gesture | Trackpad gestures | Swipe down with four fingers. |
| MC.05.03.e017 | Swipe between full-screen apps | gesture | Trackpad gestures and Magic Mouse gestures | Swipe left or right with four fingers on the trackpad or two fingers on Magic Mouse. |
| MC.05.03.e018 | System Settings | menu item | Apple menu  | Choose this to access Trackpad or Mouse settings. |
| MC.05.03.e019 | System Preferences | menu item | Apple menu  | Alternative to System Settings in the page's instructions. |
| MC.05.03.e020 | Apple menu  | menu | Mac menu bar | Menu from which the page says to choose System Settings or System Preferences. |
| MC.05.03.e021 | Trackpad | setting | System Settings or System Preferences | Trackpad settings provide gesture information and options to turn gestures off or change their type. |
| MC.05.03.e022 | Mouse | setting | System Settings or System Preferences | Mouse settings provide gesture information and options to turn gestures off or change their type. |
| MC.05.03.e023 | Accessibility settings | setting | System Settings | The page says to turn on Three finger drag in Accessibility settings. |
| MC.05.03.e024 | Need more help? | view | Below the gesture sections | Help section inviting the reader to provide more information about what is happening. |
| MC.05.03.e025 | Helpful? | control | Below the help section | Feedback prompt. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.05.03.b001 | Tap with one finger | Click. | Requires a Magic Trackpad or built-in Multi-Touch trackpad. |
| MC.05.03.b002 | Click or tap with two fingers | Secondary click (right-click). | Requires a Magic Trackpad or built-in Multi-Touch trackpad. |
| MC.05.03.b003 | Double-tap with two fingers | Zoom in and back out of a webpage or PDF. | Trackpad gesture. |
| MC.05.03.b004 | Slide two fingers up or down | Scroll. | Trackpad gesture; scrolling can be turned off in Accessibility settings. |
| MC.05.03.b005 | Pinch with two fingers | Zoom in or out. | Trackpad gesture. |
| MC.05.03.b006 | Move two fingers around each other | Rotate a photo or other item. | Trackpad gesture. |
| MC.05.03.b007 | Swipe left or right with two fingers | Show the previous or next page. | Trackpad gesture. |
| MC.05.03.b008 | Swipe left from the right edge with two fingers | Show Notification Center. | Trackpad gesture. |
| MC.05.03.b009 | Use three fingers to drag an item, then click or tap | Drag the item on screen and drop it. | Trackpad gesture; turn on Three finger drag in Accessibility settings. |
| MC.05.03.b010 | Tap with three fingers | Look up a word or take actions with dates, addresses, phone numbers, and other data. | Trackpad gesture. |
| MC.05.03.b011 | Spread your thumb and three fingers apart | Show the desktop. | Trackpad gesture. |
| MC.05.03.b012 | Pinch your thumb and three fingers together | Show installed apps. | Trackpad gesture. |
| MC.05.03.b013 | Swipe up with four fingers | Open Mission Control. | Trackpad gesture; in some versions of macOS, this gesture uses three fingers instead of four. |
| MC.05.03.b014 | Swipe down with four fingers | See all windows of the app being used. | Trackpad gesture; in some versions of macOS, this gesture uses three fingers instead of four. |
| MC.05.03.b015 | Swipe left or right with four fingers | Move between desktops and full-screen apps. | Trackpad gesture; in some versions of macOS, this gesture uses three fingers instead of four. |
| MC.05.03.b016 | Click the right side of the mouse | Secondary click (right-click). | Requires a Magic Mouse. |
| MC.05.03.b017 | Slide one finger up or down | Scroll. | Magic Mouse gesture. |
| MC.05.03.b018 | Double-tap with one finger | Zoom in and back out of a webpage or PDF. | Magic Mouse gesture. |
| MC.05.03.b019 | Double-tap with two fingers | Open Mission Control. | Magic Mouse gesture. |
| MC.05.03.b020 | Swipe left or right with two fingers | Move between desktops and full-screen apps. | Magic Mouse gesture. |
| MC.05.03.b021 | Swipe left or right with one finger | Show the previous or next page. | Magic Mouse gesture. |
| MC.05.03.b022 | Choose System Settings or System Preferences from the Apple menu , then click Trackpad or Mouse | View which gestures work with the Mac, turn a gesture off, or change the type of gesture. | Choose Trackpad for trackpad gestures or Mouse for Magic Mouse gestures. |
| MC.05.03.b023 | Use a Force Touch-capable trackpad | Force click and get haptic feedback. | Requires a trackpad that supports Force Touch. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.05.03.k001 | Tap with one finger | Click. | Trackpad gesture. |
| MC.05.03.k002 | Click or tap with two fingers | Secondary click (right-click). | Trackpad gesture. |
| MC.05.03.k003 | Double-tap with two fingers | Zoom in and back out of a webpage or PDF. | Trackpad gesture. |
| MC.05.03.k004 | Slide two fingers up or down | Scroll. | Trackpad gesture. |
| MC.05.03.k005 | Pinch with two fingers | Zoom in or out. | Trackpad gesture. |
| MC.05.03.k006 | Move two fingers around each other | Rotate a photo or other item. | Trackpad gesture. |
| MC.05.03.k007 | Swipe left or right with two fingers | Show the previous or next page. | Trackpad gesture. |
| MC.05.03.k008 | Swipe left from the right edge with two fingers | Show Notification Center. | Trackpad gesture. |
| MC.05.03.k009 | Use three fingers to drag | Drag items on screen; click or tap to drop. | Trackpad gesture; feature must be turned on in Accessibility settings. |
| MC.05.03.k010 | Tap with three fingers | Look up a word or take actions with detected data. | Trackpad gesture. |
| MC.05.03.k011 | Spread your thumb and three fingers apart | Show the desktop. | Trackpad gesture. |
| MC.05.03.k012 | Pinch your thumb and three fingers together | Show installed apps. | Trackpad gesture. |
| MC.05.03.k013 | Swipe up with four fingers | Open Mission Control. | Trackpad gesture; some macOS versions use three fingers. |
| MC.05.03.k014 | Swipe down with four fingers | See all windows of the app being used. | Trackpad gesture; some macOS versions use three fingers. |
| MC.05.03.k015 | Swipe left or right with four fingers | Move between desktops and full-screen apps. | Trackpad gesture; some macOS versions use three fingers. |
| MC.05.03.k016 | Click the right side of the mouse | Secondary click (right-click). | Magic Mouse gesture. |
| MC.05.03.k017 | Slide one finger up or down | Scroll. | Magic Mouse gesture. |
| MC.05.03.k018 | Double-tap with one finger | Zoom in and back out of a webpage or PDF. | Magic Mouse gesture. |
| MC.05.03.k019 | Double-tap with two fingers | Open Mission Control. | Magic Mouse gesture. |
| MC.05.03.k020 | Swipe left or right with two fingers | Move between desktops and full-screen apps. | Magic Mouse gesture. |
| MC.05.03.k021 | Swipe left or right with one finger | Show the previous or next page. | Magic Mouse gesture. |

System Settings paths referenced: Apple menu  > System Settings > Trackpad; Apple menu  > System Preferences > Trackpad; Apple menu  > System Settings > Mouse; Apple menu  > System Preferences > Mouse; Accessibility settings

### MC.05.04 Stage Manager

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).


#### Organize your Mac desktop with Stage Manager

T1 · OS documentation · [Apple Support](https://support.apple.com/en-us/102355) · macOS 27 text present: no

Stage Manager keeps the app being used in the center of the screen and recently used apps along the left. You can arrange windows, group apps, and change Stage Manager and desktop display settings.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.05.04.e001 | Stage Manager | setting | Desktop & Dock settings; also available in Control Center | Keeps the working window or app group centered and arranges recently used apps along the left side of the screen. |
| MC.05.04.e002 | Apple menu | menu | Mac menu bar | Named as the starting point for accessing Desktop & Dock settings. |
| MC.05.04.e003 | Open Desktop & Dock settings for me | other | Link in the instructions | Opens Desktop & Dock settings. |
| MC.05.04.e004 | Control Center | panel | Mac menu bar | Stage Manager is always available here. |
| MC.05.04.e005 | Desktop & Finder | view | Control Center | The instructions say to go here when adding Stage Manager to the menu bar. |
| MC.05.04.e006 | Add to Menu Bar | menu item | Stage Manager options in Control Center | Adds Stage Manager to the menu bar. |
| MC.05.04.e007 | Done | button | Control Center | Completes the menu bar change. |
| MC.05.04.e008 | System Settings | app | Mac | The app containing the Stage Manager settings described here. |
| MC.05.04.e009 | Desktop & Dock | view | System Settings | Contains the Desktop & Stage Manager settings. |
| MC.05.04.e010 | Desktop & Stage Manager | view | Desktop & Dock settings | Contains the settings described in this article. |
| MC.05.04.e011 | Show Items | setting | Desktop & Stage Manager | Checkboxes control whether desktop items are shown on the desktop and in Stage Manager. |
| MC.05.04.e012 | On Desktop | setting | Show Items | Controls whether desktop items are shown. |
| MC.05.04.e013 | In Stage Manager | setting | Show Items | Controls whether desktop items are shown when Stage Manager is turned on. |
| MC.05.04.e014 | Show Desktop | control | Desktop & Stage Manager | Pop-up menu for choosing when clicking the wallpaper moves windows out of the way. |
| MC.05.04.e015 | Always on Wallpaper Click | menu item | Show Desktop pop-up menu | Clicking the wallpaper moves all windows out of the way to show desktop items and widgets. |
| MC.05.04.e016 | Only in Stage Manager on Click | menu item | Show Desktop pop-up menu | Clicking the wallpaper moves all windows out of the way to show desktop items and widgets when Stage Manager is turned on. |
| MC.05.04.e017 | Show recent apps in Stage Manager | setting | Desktop & Stage Manager | Controls whether recently used apps are shown along the left side of the screen. |
| MC.05.04.e018 | Show windows from an application | control | Desktop & Stage Manager | Pop-up menu for choosing how an app's available windows are shown when switching to it. |
| MC.05.04.e019 | All at Once | menu item | Show windows from an application pop-up menu | Shows all available windows for an app when switching to it. |
| MC.05.04.e020 | One at a Time | menu item | Show windows from an application pop-up menu | Shows only the most recently used window for an app when switching to it. |
| MC.05.04.e021 | recently used apps | view | Left side of the screen in Stage Manager | Apps are arranged here for quick access; the list can be hidden by a setting. |
| MC.05.04.e022 | window | window | Center of the screen | The window being worked with is positioned in the center. |
| MC.05.04.e023 | group of apps | view | Center of the screen in Stage Manager | Multiple apps can be arranged together; switching to the group opens its apps in the center. |
| MC.05.04.e024 | desktop items and widgets | view | Desktop | Can be shown by clicking the wallpaper according to the selected Show Desktop option. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.05.04.b001 | Turn Stage Manager on | The working app is kept front and center, and recently used apps are arranged along the left side. | Stage Manager must be turned on. |
| MC.05.04.b002 | Turn Stage Manager off | You can return to traditional windows. | No further condition stated. |
| MC.05.04.b003 | Click an app on the left side of the screen | Switches to that app. | Stage Manager is in use and the app list is visible. |
| MC.05.04.b004 | Reposition, resize, or overlap windows | Arranges windows to suit the workflow. | No further condition stated. |
| MC.05.04.b005 | Drag an app from the left side of the screen to the center | Adds the app to a group of apps. | Stage Manager is in use. |
| MC.05.04.b006 | Drag an app to the left side of the screen | Removes the app from its group. | Stage Manager is in use. |
| MC.05.04.b007 | Switch to a group | All apps in the group open in the center of the screen. | The apps have been arranged as a group in Stage Manager. |
| MC.05.04.b008 | Turn off “Show recent apps in Stage Manager” | The list of recently used apps is hidden; moving the pointer to the left edge briefly shows them. | The setting is off. |
| MC.05.04.b009 | Move the pointer to the left edge of the screen | Briefly shows the recently used apps. | “Show recent apps in Stage Manager” is off. |
| MC.05.04.b010 | Choose Add to Menu Bar | Shows Stage Manager in the menu bar as well as making it available in Control Center. | The Stage Manager options are accessed in Control Center. |
| MC.05.04.b011 | Select “On Desktop” | Shows desktop items. | This is a checkbox under Show Items. |
| MC.05.04.b012 | Select “In Stage Manager” | Shows desktop items when Stage Manager is turned on. | This is a checkbox under Show Items. |
| MC.05.04.b013 | Turn off “In Stage Manager,” then click the desktop | Desktop items, which are hidden, are shown for access. | The option is turned off. |
| MC.05.04.b014 | Choose “Always on Wallpaper Click,” then click the wallpaper | Moves all windows out of the way to show desktop items and widgets. | That option is selected in the Show Desktop pop-up menu. |
| MC.05.04.b015 | Choose “Only in Stage Manager on Click,” then click the wallpaper | Moves all windows out of the way to show desktop items and widgets. | That option is selected and Stage Manager is turned on. |
| MC.05.04.b016 | Turn “Show recent apps in Stage Manager” on | Shows recently used apps. | Stage Manager settings. |
| MC.05.04.b017 | Turn “Show recent apps in Stage Manager” off | Hides recently used apps; moving the pointer to the left edge briefly shows them. | Stage Manager settings. |
| MC.05.04.b018 | Choose “All at Once” | Shows all available windows for an app when switching to it. | Selected in the Show windows from an application pop-up menu. |
| MC.05.04.b019 | Choose “One at a Time” | Shows only the app's most recently used window when switching to it. | Selected in the Show windows from an application pop-up menu. |
| MC.05.04.b020 | Click the app on the left again | Opens the next available window for that app. | “One at a Time” is selected and you want to switch to a different window. |
| MC.05.04.b021 | Try to turn on Stage Manager when it cannot be turned on | The article says to choose the Apple menu. | The reason Stage Manager cannot be turned on is not specified. |

System Settings paths referenced: System Settings > Desktop & Dock > Desktop & Stage Manager; Apple menu > Desktop & Dock settings (the article does not provide a complete menu path)

#### Organize your Mac desktop with Stage Manager

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchl534ba392/27/mac/27) · macOS 27 text present: no

The page explains how to turn Stage Manager on or off, work with app windows and groups, show Stage Manager in the menu bar, and change Stage Manager settings.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.05.04.e025 | Apple menu | menu | Mac menu bar | The page refers to choosing this menu when turning Stage Manager on or off and when Stage Manager cannot be turned on. |
| MC.05.04.e026 | Open Desktop & Dock settings for me | other | Link in the turn Stage Manager on or off instructions and the settings instructions | Link text for opening Desktop & Dock settings. |
| MC.05.04.e027 | Stage Manager | setting | Desktop & Stage Manager settings; also available in Control Center | The feature that keeps the working app centered and recently used apps arranged along the left side. |
| MC.05.04.e028 | Control Center | other | Mac menu bar | The page says Stage Manager is always available here. |
| MC.05.04.e029 | Desktop & Finder | other | Control Center instructions | The page says to go here and move the pointer over Stage Manager. |
| MC.05.04.e030 | Add to Menu Bar | menu item | Menu opened from the Stage Manager controls in Control Center | The page says to choose this to also show Stage Manager in the menu bar. |
| MC.05.04.e031 | Done | button | Control Center instructions | The page says to click this after choosing Add to Menu Bar. |
| MC.05.04.e032 | System Settings | other | Mac | The app the page directs the user to open for changing Stage Manager settings. |
| MC.05.04.e033 | Desktop & Dock | other | System Settings | The settings section the page directs the user to open. |
| MC.05.04.e034 | Desktop & Stage Manager | other | Desktop & Dock settings | The subsection named in the Stage Manager settings instructions. |
| MC.05.04.e035 | Show Items | view | Desktop & Stage Manager settings | The section containing the desktop item checkboxes. |
| MC.05.04.e036 | On Desktop | setting | Show Items section | Checkbox for showing desktop items. |
| MC.05.04.e037 | In Stage Manager | setting | Show Items section | Checkbox for showing desktop items when Stage Manager is turned on. |
| MC.05.04.e038 | Show Desktop | control | Desktop & Stage Manager settings | Pop-up menu for choosing how clicking the wallpaper reveals desktop items and widgets. |
| MC.05.04.e039 | Always on Wallpaper Click | menu item | Show Desktop pop-up menu | Option described as moving all windows out of the way when the wallpaper is clicked. |
| MC.05.04.e040 | Only in Stage Manager on Click | menu item | Show Desktop pop-up menu | Option described as moving all windows out of the way on wallpaper click only when Stage Manager is turned on. |
| MC.05.04.e041 | Show recent apps in Stage Manager | setting | Desktop & Stage Manager settings | Setting that controls whether recently used apps are shown at the left side of the screen. |
| MC.05.04.e042 | Show windows from an application | control | Desktop & Stage Manager settings | Pop-up menu for choosing how windows appear when switching to an app. |
| MC.05.04.e043 | All at Once | menu item | Show windows from an application pop-up menu | Option that shows all available windows for an app when switching to it. |
| MC.05.04.e044 | One at a Time | menu item | Show windows from an application pop-up menu | Option that shows only the most recently used window for an app when switching to it. |
| MC.05.04.e045 | Recently used apps | view | Left side of the screen in Stage Manager | Apps arranged for quick access. |
| MC.05.04.e046 | Working app window | window | Center of the screen | The window for the app being worked with. |
| MC.05.04.e047 | App group | view | Center of the screen | A group of apps that can be arranged together and opened together. |
| MC.05.04.e048 | Desktop items and widgets | view | Mac desktop | Items and widgets that can be shown by clicking the wallpaper, according to the selected Show Desktop option. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.05.04.b022 | Turn Stage Manager on. | The app being worked with is positioned in the center of the screen, and recently used apps are arranged along the left side. | When Stage Manager is on and Show recent apps in Stage Manager is enabled. |
| MC.05.04.b023 | Turn Stage Manager off. | The user can return to traditional windows. | The page describes switching between Stage Manager and traditional windows. |
| MC.05.04.b024 | Click an app on the left side of the screen. | Switches to that app. | While using Stage Manager and the app list is visible. |
| MC.05.04.b025 | Reposition, resize, or overlap windows. | Arranges the windows to suit the user's workflow. | While using Stage Manager. |
| MC.05.04.b026 | Drag an app from the left side of the screen to a group of apps in the center. | Adds the app to the group. | While using Stage Manager. |
| MC.05.04.b027 | Switch to a group. | All apps in the group open in the center of the screen. | The apps have been arranged as a group in Stage Manager. |
| MC.05.04.b028 | Drag an app to the left side of the screen. | Removes the app from its group. | While using Stage Manager. |
| MC.05.04.b029 | Move the pointer to the left edge of the screen. | Briefly shows the recently used apps. | Show recent apps in Stage Manager is turned off, so the app list is hidden. |
| MC.05.04.b030 | Move the pointer over Stage Manager. | Positions the pointer over the Stage Manager control. | In Control Center, after going to Desktop & Finder. |
| MC.05.04.b031 | Choose Add to Menu Bar, then click Done. | Shows Stage Manager in the menu bar as well as keeping it available in Control Center. | While following the Control Center instructions. |
| MC.05.04.b032 | Select the On Desktop checkbox. | Shows desktop items. | In the Show Items section of Desktop & Stage Manager settings. |
| MC.05.04.b033 | Select the In Stage Manager checkbox. | Shows desktop items when Stage Manager is turned on. | In the Show Items section of Desktop & Stage Manager settings. |
| MC.05.04.b034 | Turn off the In Stage Manager option, then click the desktop. | Desktop items are hidden until the desktop is clicked; clicking shows the items for access. | The page gives this behavior when the option is turned off. |
| MC.05.04.b035 | Click the wallpaper. | Moves all windows out of the way to show desktop items and widgets. | Show Desktop is set to Always on Wallpaper Click. |
| MC.05.04.b036 | Click the wallpaper. | Moves all windows out of the way to show desktop items and widgets. | Show Desktop is set to Only in Stage Manager on Click, and Stage Manager is turned on. |
| MC.05.04.b037 | Turn Show recent apps in Stage Manager off. | Hides recently used apps. | In Desktop & Stage Manager settings. |
| MC.05.04.b038 | Switch to an app. | Shows all available windows for that app. | Show windows from an application is set to All at Once. |
| MC.05.04.b039 | Switch to an app. | Shows only the most recently used window for that app. | Show windows from an application is set to One at a Time. |
| MC.05.04.b040 | Click the app on the left again. | Opens the next available window for that app. | The page specifies this when the Show windows from an application option is turned off. |

System Settings paths referenced: System Settings > Desktop & Dock > Desktop & Stage Manager

#### Independent reports (MC.05.04)

| ID | Kind | Area | Claim | Source |
|---|---|---|---|---|
| MC.05.04.x01 | workaround | Stage Manager | A commenter says Apple TV worked correctly again after they disabled Stage Manager. | T5 [Reddit r/MacOS](https://www.reddit.com/r/MacOS/comments/1wlax16/mission_control_expose_is_broken_in_macos_27/) |
| MC.05.04.x02 | behavior | Stage Manager | The implementation notes report that, with Stage Manager enabled, macOS draws no Show Desktop overlay and reveals the desktop by removing an event shield instead. This was described as a measured observation. | T2 [GitHub lwouis/alt-tab-macos](https://github.com/lwouis/alt-tab-macos/commit/cbd8e912dded9a7f137647c1f496bfe06ed875df) |
| MC.05.04.x03 | behavior | Stage Manager | Stage Manager divides the screen into a central stage and a left strip of thumbnails for recent apps; selecting a thumbnail brings that app onto the stage and moves the current one to the strip. | T4 [Lets Suite](https://www.lets-suite.com/blog/how-to-use-stage-manager-mac) |
| MC.05.04.x04 | behavior | Stage Manager | A thumbnail represents a set of one or more windows that move between the stage and strip together. | T4 [Lets Suite](https://www.lets-suite.com/blog/how-to-use-stage-manager-mac) |
| MC.05.04.x05 | behavior | Stage Manager | Stage Manager can be toggled in System Settings under Desktop & Dock, in the Desktop & Stage Manager section. | T4 [Lets Suite](https://www.lets-suite.com/blog/how-to-use-stage-manager-mac) |
| MC.05.04.x06 | behavior | Stage Manager | The article says Stage Manager requires Displays have separate Spaces to be enabled. | T4 [Lets Suite](https://www.lets-suite.com/blog/how-to-use-stage-manager-mac) |
| MC.05.04.x07 | behavior | Stage Manager | Dragging an app thumbnail from the strip onto the stage adds that app to the current set rather than replacing the windows already there. | T4 [Lets Suite](https://www.lets-suite.com/blog/how-to-use-stage-manager-mac) |
| MC.05.04.x08 | behavior | Stage Manager | To add one particular window to a set, bring up the app's windows by clicking its thumbnail, then drag the chosen window onto the stage. | T4 [Lets Suite](https://www.lets-suite.com/blog/how-to-use-stage-manager-mac) |
| MC.05.04.x09 | behavior | Stage Manager | Dragging a window back to the strip removes it from its set and makes it its own set. | T4 [Lets Suite](https://www.lets-suite.com/blog/how-to-use-stage-manager-mac) |
| MC.05.04.x10 | behavior | Stage Manager | Stage Manager remembers a set's window arrangement for the next time the set is selected. | T4 [Lets Suite](https://www.lets-suite.com/blog/how-to-use-stage-manager-mac) |
| MC.05.04.x11 | behavior | Stage Manager | Show recent apps in Stage Manager is on by default. When turned off, the strip appears only when the pointer moves to the left edge. | T4 [Lets Suite](https://www.lets-suite.com/blog/how-to-use-stage-manager-mac) |
| MC.05.04.x12 | behavior | Stage Manager | Show windows from an application is set to All at Once by default; One at a Time brings only the most recent window of an app onto the stage. | T4 [Lets Suite](https://www.lets-suite.com/blog/how-to-use-stage-manager-mac) |
| MC.05.04.x13 | behavior | Stage Manager | Show items: In Stage Manager is off by default; turning it on shows desktop files that Stage Manager otherwise hides. | T4 [Lets Suite](https://www.lets-suite.com/blog/how-to-use-stage-manager-mac) |
| MC.05.04.x14 | behavior | Stage Manager | Click wallpaper to reveal desktop has Always and Only in Stage Manager options; with Stage Manager on, clicking the wallpaper sweeps windows aside to show the desktop. | T4 [Lets Suite](https://www.lets-suite.com/blog/how-to-use-stage-manager-mac) |
| MC.05.04.x15 | behavior | Stage Manager | With more than one display, each display runs its own Stage Manager with its own strip, and windows can be dragged between displays. | T4 [Lets Suite](https://www.lets-suite.com/blog/how-to-use-stage-manager-mac) |
| MC.05.04.x16 | behavior | Stage Manager | The article says Stage Manager can feel less responsive on older Macs with many windows open, due to animation and live thumbnails; it suggests trying fewer sets or turning it off for heavy work. | T4 [Lets Suite](https://www.lets-suite.com/blog/how-to-use-stage-manager-mac) |
| MC.05.04.x17 | behavior | Stage Manager | The article reports no confirmed Stage Manager changes in macOS 27 and describes it as a tidiness mode rather than a window-placement tool. | T4 [Lets Suite](https://www.lets-suite.com/blog/macos-27-window-management) |

## 14.6 MC.06 Spotlight


### MC.06.01 Search

Status: **PARTIAL**. Spotlight guide captured; Spotlight menu-bar icon status conflicts (GAP-B5-05).


#### Find what you need with Spotlight on Mac - Apple Support

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp1008/27/mac/27) · macOS 27 text present: no

Spotlight opens with Command-Space bar. Type to see search results, then open, preview, locate, or act on results. Spotlight can also calculate, convert units, define words, search clipboard history, and—when Siri AI is turned on—accept questions and requests for Siri.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.06.01.e001 | Spotlight | window | On Mac; opened with Command-Space bar | Search interface for finding items and using the listed search and action features. Its window can be repositioned or resized by dragging. |
| MC.06.01.e002 | Siri AI (Beta) | setting | Referenced in the Spotlight introduction | When turned on, enables starting a Siri conversation from Spotlight. |
| MC.06.01.e003 | search results | view | In Spotlight | Results appear as the user types, with the best match at the top. |
| MC.06.01.e004 | Quick Actions | view | For a selected Spotlight result | Actions available for a result; availability depends on what is selected. |
| MC.06.01.e005 | Quick Look | other | For a Spotlight result | Previews a file without opening it. |
| MC.06.01.e006 | Finder | other | When locating a selected file result | The article says a file can be located in the Finder. |
| MC.06.01.e007 | Dictionary | other | Referenced in the word lookup description | The article says definitions can be viewed in Spotlight without opening Dictionary. |
| MC.06.01.e008 | Clipboard history | view | In Spotlight search | Searches text and files previously copied to the clipboard. |
| MC.06.01.e009 | Notification Center | other | Related learning links | Described as a place for notifications from apps such as Messages and Mail. |
| MC.06.01.e010 | Control Center | other | Related learning links | Described as a place to quickly change settings such as screen brightness, Wi-Fi, and Do Not Disturb. |
| MC.06.01.e011 | Siri | other | Spotlight and related learning links | Can answer questions and take action; with Siri AI, questions and requests can be entered from Spotlight. |
| MC.06.01.e012 | Messages | other | Example app in the Notification Center description | Named as an example of an app whose notifications appear in Notification Center. |
| MC.06.01.e013 | Mail | other | Example app in the Notification Center description | Named as an example of an app whose notifications appear in Notification Center. |
| MC.06.01.e014 | Safari | other | Closing suggested learning links | Described as the built-in browser on Mac. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.06.01.b001 | Press Command-Space bar | Open Spotlight. | On Mac. |
| MC.06.01.b002 | Start typing in Spotlight | Search results appear instantly, with the best match at the top. | — |
| MC.06.01.b003 | Press Return on a result | Open the result. | — |
| MC.06.01.b004 | Use the arrow keys to navigate the results list | Navigate the list of results. | — |
| MC.06.01.b005 | Drag the Spotlight window | Reposition or resize the window. | — |
| MC.06.01.b006 | Double-click a result | Open the result. | — |
| MC.06.01.b007 | Press the Up Arrow | Recall a previous search. | In Spotlight. |
| MC.06.01.b008 | Select a file result and hold Command | Locate the file in the Finder. | The selected result is a file. |
| MC.06.01.b009 | Press Space bar on a result | Preview the file without opening it. | Works for documents, images, PDFs, and videos. |
| MC.06.01.b010 | Select a result and press Tab | Reveal Quick Actions. | Available actions depend on what is selected. |
| MC.06.01.b011 | Type a math expression, such as “15% of 85,” “120 × 1.08,” or “√144” | The answer appears instantly in the search field. | — |
| MC.06.01.b012 | Type a unit conversion, such as “32°F in Celsius,” “5 miles in km,” or “€200 in USD” | Spotlight converts the units. | The article lists temperature, distance, weight, currency, and more. |
| MC.06.01.b013 | Type a word | See its definition in Spotlight. | Without opening Dictionary. |
| MC.06.01.b014 | Search in Spotlight | Find text and files previously copied to the clipboard. | The text or files were previously copied to the clipboard. |
| MC.06.01.b015 | Enter a question or make a request, then press Return | Ask Siri from Spotlight. | Siri AI is turned on. |
| MC.06.01.b016 | Turn on Siri AI (Beta) | Spotlight becomes a place to start a conversation with Siri, including asking for personal context, getting broad world knowledge, or taking actions across apps. | Siri AI is available in beta and requires an Apple Intelligence-enabled device set to a supported language; it is available in English to start. |
| MC.06.01.b017 | Use Notification Center | View notifications from apps such as Messages and Mail in one place. | — |
| MC.06.01.b018 | Use Control Center | Quickly change settings such as screen brightness, connect to Wi-Fi, and turn on Do Not Disturb. | — |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.06.01.k001 | Command-Space bar | Open Spotlight. | On Mac. |
| MC.06.01.k002 | Return | Open a selected result. | In Spotlight. |
| MC.06.01.k003 | Arrow keys | Navigate the results list. | In Spotlight. |
| MC.06.01.k004 | Up Arrow | Recall a previous search. | In Spotlight. |
| MC.06.01.k005 | Command (held) | Locate the selected file in the Finder. | With a file result selected. |
| MC.06.01.k006 | Space bar | Preview a file without opening it. | On a result; documents, images, PDFs, and videos are specified. |
| MC.06.01.k007 | Tab | Reveal Quick Actions. | With a result selected. |
| MC.06.01.k008 | Return | Submit a question or request to Siri. | In Spotlight with Siri AI turned on. |

macOS 27 statements on this page:

- MC.06.01.n01: The article states that Siri AI is available in beta in macOS 27. It does not explicitly describe a Spotlight feature as newly introduced or changed in macOS 27.

#### Independent reports (MC.06.01)

| ID | Kind | Area | Claim | Source |
|---|---|---|---|---|
| MC.06.01.x01 | changed | Spotlight | Beginning with macOS 27.0 beta 3, Spotlight was removed from the menu bar in favor of Siri for users who joined the waitlist. | T3 [Wikipedia](https://en.wikipedia.org/wiki/MacOS_Golden_Gate) |
| MC.06.01.x02 | measurement | Spotlight | After installing new betas, Spotlight indexing finished quickly; searching the index was faster, with results appearing as quickly as the author could type. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/10/) |
| MC.06.01.x03 | behavior | Spotlight | Apple rebuilt the search foundation used by Spotlight, Photos, and Mail to make search more up to date and complete. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/10/) |
| MC.06.01.x04 | behavior | Spotlight | Spotlight offers more relevant search suggestions. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/10/) |
| MC.06.01.x05 | measurement | other | Native apps launched faster, though the author noted that not every app launched instantly. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/10/) |
| MC.06.01.x06 | measurement | other | Scrolling and animations felt smoother, especially when the author first installed Golden Gate. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/10/) |
| MC.06.01.x07 | changed | Spotlight | Siri and Spotlight have been merged into one interface for typing to find things. | T4 [Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/) |
| MC.06.01.x08 | changed | Spotlight | Spotlight suggestions and search in Apple Mail are improved. | T4 [Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/) |
| MC.06.01.x09 | behavior | Spotlight | Siri AI is labeled as a beta feature in macOS 27.0. | T4 [Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/) |
| MC.06.01.x10 | changed | Spotlight | Spotlight is being rebuilt with a faster, more efficient, and more stable index architecture intended to speed up indexing. | T4 [Cult of Mac](https://www.cultofmac.com/news/macos-27-new-features-beyond-siri-ai) |
| MC.06.01.x11 | changed | Spotlight | The new indexing architecture will also power search in Mail. | T4 [Cult of Mac](https://www.cultofmac.com/news/macos-27-new-features-beyond-siri-ai) |
| MC.06.01.x12 | new | Spotlight | Users will be able to trigger Siri AI through Spotlight. | T4 [Cult of Mac](https://www.cultofmac.com/news/macos-27-new-features-beyond-siri-ai) |
| MC.06.01.x13 | behavior | Spotlight | Spotlight retains a Screenshot category filter. Users can press Space bar on a result for Quick Look or hold Command to locate it in Finder. | T4 [Pizazoo](https://pizazoo.com/blog/macos-27-screenshot-features/) |
| MC.06.01.x14 | new | Spotlight | With Siri AI enabled, Spotlight is also a starting point for Siri conversations. | T4 [Pizazoo](https://pizazoo.com/blog/macos-27-screenshot-features/) |
| MC.06.01.x15 | behavior | Spotlight | To apply spacing immediately, Thaw restarts apps that own menu bar items. Spotlight, the input menu, Dock, and Time Machine are restarted through launchctl kickstart so launchd remains their launching parent. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.06.01.x16 | changed | Spotlight | Spotlight is now a place to search the Mac or ask Siri AI questions using a new bar; users can converse with Siri, find information in supported apps, and take supported actions. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MC.06.01.x17 | new | Spotlight | The new Siri AI is the default for the Siri app and the Command-Space bar, but users can switch to ChatGPT from a context menu. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MC.06.01.x18 | new | other | A new Show Borders accessibility option makes the boundaries of interface elements easier to see. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MC.06.01.x19 | new | other | Siri's expressiveness and speaking pace can be adjusted with sliders in a new Voice section of System Settings. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MC.06.01.x20 | changed | Spotlight | The post describes Siri and Spotlight as merged into one, with faster search-content indexing, better search integration between apps, more relevant suggestions, and faster indexing of shortcuts and actions. | T5 [MacRumors Forums](https://forums.macrumors.com/threads/macos-27-all-the-little-things.2483520/) |
| MC.06.01.x21 | new | Spotlight | The Siri AI app can be opened from Spotlight. | T4 [Tom's Guide](https://www.tomsguide.com/computing/macos/macos-golden-gate-review) |
| MC.06.01.x22 | changed | Spotlight | The new Siri AI is available in Spotlight, and Spotlight indexing is faster, particularly for shortcuts and actions. | T4 [9to5Mac](https://9to5mac.com/2026/09/14/macos-27-golden-gate-now-available-here-is-everything-new/) |
| MC.06.01.x23 | changed | Spotlight | The underlying system for Spotlight suggestions has been tweaked. | T4 [9to5Mac](https://9to5mac.com/2026/09/14/macos-27-golden-gate-now-available-here-is-everything-new/) |

### MC.06.02 Actions & quick keys

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).


#### Take actions and shortcuts in Spotlight on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchl4953dfeb/27/mac/27) · macOS 27 text present: yes

Spotlight can search for and run actions or shortcuts, with optional quick keys for faster access. Apple Intelligence and Siri AI add further capabilities, subject to stated device, language, region, and usage conditions.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.06.02.e001 | Spotlight | view | Mac | Search interface used to find and run actions or shortcuts. |
| MC.06.02.e002 | search field | control | Spotlight | Enter a search, a request for Siri, or a quick key; a contact name can follow a quick key. |
| MC.06.02.e003 | Ask Siri | menu item | Spotlight results below the search field | Select to ask Siri a question or make a request. |
| MC.06.02.e004 | Use Model | menu item | Spotlight action suggestions | Asks an LLM to answer a question. Available only with Apple Intelligence. |
| MC.06.02.e005 | Ask a question | menu item | Spotlight action suggestions | Asks Siri a question. Available only when Siri AI is turned on. |
| MC.06.02.e006 | Add File to Note | menu item | Spotlight action suggestions | Quickly adds a file on the computer to a note in Notes. |
| MC.06.02.e007 | Change Case | menu item | Spotlight action suggestions | Changes the case of copied text. |
| MC.06.02.e008 | Create Image | menu item | Spotlight action suggestions | Generates an image from a description. |
| MC.06.02.e009 | Start FaceTime Call | menu item | Spotlight action suggestions | Makes an audio or video FaceTime call to a chosen person. |
| MC.06.02.e010 | Random Number | menu item | Spotlight action suggestions | Picks a random number when given a minimum and maximum. |
| MC.06.02.e011 | Recognize Music | menu item | Spotlight action suggestions | Names the currently playing song. |
| MC.06.02.e012 | Remove Image Background | menu item | Spotlight action suggestions | Removes the background from a named image. |
| MC.06.02.e013 | Send Email | menu item | Spotlight action suggestions | Sends an email message to a chosen recipient. |
| MC.06.02.e014 | Translate Text | menu item | Spotlight action suggestions | Translates selected text from a detected language to a chosen language. |
| MC.06.02.e015 | Add quick keys | button | Next to an action in Spotlight | Starts assigning a quick key to an action. |
| MC.06.02.e016 | quick key field | control | Beside an action with an assigned quick key | Click to edit the assigned quick key. |
| MC.06.02.e017 | Siri AI (Beta) | setting | Not specified | When turned on, allows requests and questions through conversation in Spotlight. |
| MC.06.02.e018 | Notes | other | App | The app where Add File to Note adds a file. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.06.02.b001 | Press Command-Space bar. | Spotlight opens. | On a Mac. |
| MC.06.02.b002 | Enter a search for a specific action or browse the suggestions, then double-click an action. | The selected action opens. | In Spotlight. |
| MC.06.02.b003 | Click Ask Siri in the results below the search field. | Siri is asked the request typed in Spotlight. | Type the request in Spotlight first. |
| MC.06.02.b004 | Choose Use Model and ask a question. | An LLM answers the question. | Apple Intelligence is available and enabled; usage limits may apply. |
| MC.06.02.b005 | Choose Ask a question. | Siri is asked a question. | Siri AI is turned on. |
| MC.06.02.b006 | Choose Add File to Note. | A file on the computer is added to a note in Notes. | The action may require filling in blanks. |
| MC.06.02.b007 | Choose Change Case. | The case of copied text changes. | Copied text is available. |
| MC.06.02.b008 | Choose Create Image and provide a description. | An image is generated. | The action may require filling in blanks. |
| MC.06.02.b009 | Choose Start FaceTime Call and provide a person. | A FaceTime audio or video call is made. | The action may require filling in blanks. |
| MC.06.02.b010 | Choose Random Number and provide a minimum and maximum. | A random number is picked. | A minimum and maximum are given. |
| MC.06.02.b011 | Choose Recognize Music. | The currently playing song is named. | A song is currently playing. |
| MC.06.02.b012 | Choose Remove Image Background and name an image. | The image background is removed. | The action may require filling in blanks. |
| MC.06.02.b013 | Choose Send Email and provide a recipient. | An email message is sent. | The action may require filling in blanks. |
| MC.06.02.b014 | Choose Translate Text and provide the text and target language. | Selected text is translated from a detected language to the chosen language. | The action may require filling in blanks. |
| MC.06.02.b015 | Choose an action by mistake, then press Escape. | Spotlight returns to search. | In Spotlight. |
| MC.06.02.b016 | Fill in any blanks required by an action, then press Return. | The action or shortcut runs automatically. | After choosing an action or shortcut. |
| MC.06.02.b017 | Click the action-browsing icon or press Command-3. | The search is narrowed to actions. | In Spotlight; the source does not provide a visible label for the icon. |
| MC.06.02.b018 | Click Add quick keys next to an action, then enter a keyboard shortcut. | The shortcut is assigned as a quick key for running the action. | In Spotlight. |
| MC.06.02.b019 | Run an action once. | A suggested quick key is automatically assigned. | For the action. |
| MC.06.02.b020 | Click an assigned quick key field, make a change, then press Return. | The quick key is edited. | In Spotlight. |
| MC.06.02.b021 | Type an assigned quick key, such as ft. | Spotlight prompts for a contact in the search field. | The quick key is assigned to Start FaceTime Call. |
| MC.06.02.b022 | Type a quick key followed by a contact name, such as ft Ashley, then press Return. | A FaceTime call starts. | The quick key is assigned to Start FaceTime Call. |
| MC.06.02.b023 | Ask a question or make a request conversationally in the Spotlight search field. | Siri can take action across apps. | Siri AI (Beta) is turned on; availability and usage limits apply. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.06.02.k001 | Command-Space bar | Open Spotlight. | On a Mac. |
| MC.06.02.k002 | Command-3 | Narrow the Spotlight search to actions. | When searching or browsing action suggestions. |
| MC.06.02.k003 | Escape | Return to search. | After choosing an action by mistake. |
| MC.06.02.k004 | Return | Run the action or shortcut automatically. | After filling any required blanks. |
| MC.06.02.k005 | Return | Save the edited quick key. | After changing an assigned quick key. |
| MC.06.02.k006 | Return | Start the FaceTime call. | After entering a contact after the ft quick key. |
| MC.06.02.k007 | ft | Invoke Start FaceTime Call and prompt for a contact. | Example quick key; it must be assigned to the action. |
| MC.06.02.k008 | ft Ashley | Start a FaceTime call to Ashley after pressing Return. | Example of a quick key followed by a contact name. |

macOS 27 statements on this page:

- MC.06.02.n01: Apple Intelligence on macOS 27 is not available in all languages or regions; usage limits may apply.
- MC.06.02.n02: Siri AI is available in beta on macOS 27 or later on an Apple Intelligence-enabled MacBook Neo (A18 Pro) or Mac model with M1 or later, with Siri and the device language set to a supported language; it is available in English to start, and usage limits may apply.
- MC.06.02.n03: The page states that Siri AI is available in beta in macOS 27, but does not describe a comparison with earlier macOS versions.

### MC.06.03 Intelligence/Siri integration

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).

Also covered by: [Find what you need with Spotlight on Mac - Apple Support](https://support.apple.com/guide/mac-help/mchlp1008/27/mac/27), [Take actions and shortcuts in Spotlight on Mac](https://support.apple.com/guide/mac-help/mchl4953dfeb/27/mac/27)


## 14.7 MC.07 Control Center & Notification Center


### MC.07.01 Modules & customization

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).


#### Use Control Center on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchl50f94f8f/27/mac/27) · macOS 27 text present: no

Control Center provides quick access to Mac settings and features. You can customize its items, move copies to the menu bar, and view information about apps using privacy-sensitive resources.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.07.01.e001 | Control Center | panel | Menu bar | Provides quick access to settings and features. |
| MC.07.01.e002 | AirDrop | control | Control Center | An example of a feature that can be turned on or off. |
| MC.07.01.e003 | Wi-Fi | control | Control Center | Listed as an example of a setting accessible from Control Center. |
| MC.07.01.e004 | Focus | control | Control Center | Listed as an example of a setting accessible from Control Center. |
| MC.07.01.e005 | Accessibility shortcuts | control | Available to add to Control Center | An item that can be added to Control Center. |
| MC.07.01.e006 | Battery status | control | Available to add to Control Center | An item that can be added to Control Center. |
| MC.07.01.e007 | Fast user switching | control | Available to add to Control Center | An item that can be added to Control Center. |
| MC.07.01.e008 | Menu bar | view | Top of the Mac screen | You can add copies of Control Center items to it. |
| MC.07.01.e009 | Privacy indicators | indicator | To the right of Control Center | Can be dots or arrows. |
| MC.07.01.e010 | Orange dot | indicator | To the right of Control Center | Mentioned as an example of a privacy indicator; the supplied text does not explain its meaning. |
| MC.07.01.e011 | App-activity field | field | At the top of the Control Center window, when present | Shows which apps are using the microphone, location, camera, or system audio. |
| MC.07.01.e012 | Privacy window | window | Opened from the app-activity field | May contain additional information. |
| MC.07.01.e013 | Sound | slider | Control Center | Can be dragged to adjust Mac volume. |
| MC.07.01.e014 | Bluetooth | control | Control Center | An example of a feature that can be turned on or off. |
| MC.07.01.e015 | Edit Controls | button | Control Center | Used to customize Control Center. |
| MC.07.01.e016 | Sidebar categories | view | Edit Controls window | Categories can be selected to narrow the available items. |
| MC.07.01.e017 | Done | button | Edit Controls window | Used after customizing Control Center. |
| MC.07.01.e018 | Copy to Menu Bar | menu item | Context menu for a Control Center item | Copies an item to the menu bar. |
| MC.07.01.e019 | Add to Menu Bar | menu item | Context menu for a Control Center item | Adds an item to the menu bar. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.07.01.b001 | Open Control Center | A field at the top of the window may show which apps are using the microphone, location, camera, or system audio. | The field may not be present. |
| MC.07.01.b002 | Click the app-activity field | Opens the Privacy window, which may have additional information. | The field is present. |
| MC.07.01.b003 | Drag a slider in Control Center | Increases or decreases a setting; dragging the Sound slider adjusts Mac volume. | — |
| MC.07.01.b004 | Click a feature icon in Control Center | Turns the feature on or off; AirDrop and Bluetooth are examples. | — |
| MC.07.01.b005 | Control-click a frequently used item | Shows the menu items “Copy to Menu Bar” or “Add to Menu Bar” to keep a copy of the item handy. | The item is in Control Center. |
| MC.07.01.b006 | Click Edit Controls | Opens the controls customization interface. | Control Center is open. |
| MC.07.01.b007 | Drag an item from the window to a position in Control Center | Adds the item to Control Center. | The controls customization interface is open. |
| MC.07.01.b008 | Drag an item out of Control Center | Removes the item from Control Center. | The controls customization interface is open. |
| MC.07.01.b009 | Drag an item to a different position | Rearranges the item in Control Center. | The controls customization interface is open. |
| MC.07.01.b010 | Click a category in the sidebar | Narrows the choice of items. | The controls customization interface is open. |
| MC.07.01.b011 | Control-click an item in Control Center and choose a different size | Resizes the item. | — |
| MC.07.01.b012 | Click Done | Finishes the customization steps. | The controls customization interface is open. |

#### Independent reports (MC.07.01)

| ID | Kind | Area | Claim | Source |
|---|---|---|---|---|
| MC.07.01.x01 | changed | Control Center | The Control Center has a consistent closing animation. | T3 [Wikipedia](https://en.wikipedia.org/wiki/MacOS_Golden_Gate) |
| MC.07.01.x02 | behavior | Control Center | Stage Manager can be turned on or off using its button in Control Center. | T4 [Lets Suite](https://www.lets-suite.com/blog/how-to-use-stage-manager-mac) |
| MC.07.01.x03 | behavior | Control Center | The pull request reports that Control Centre opens from an Accessibility press while menu-bar items remain concealed, unlike the clock, battery, and Wi-Fi items. | T2 [GitHub jordanbaird/Ice](https://github.com/jordanbaird/Ice/pull/995) |
| MC.07.01.x04 | behavior | Control Center | The setting Replace Control Center items puts Thaw icons in place of Apple's items, and those Thaw icons can be moved and hidden. | T2 [GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md) |
| MC.07.01.x05 | behavior | Control Center | In Golden Gate Preview One, some system items, especially Control Center widgets, cannot be hidden, though they can be moved. A later preview notes Bartender cannot show or hide items disabled in Control Center. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.07.01.x06 | behavior | Control Center | In Golden Gate Preview One, the Shortcuts menu item is labelled “Control Center” and cannot be moved. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.07.01.x07 | behavior | Control Center | Fast User Switching can be added to Control Center using Add Controls > Fast User Switching > Add to Control Center. | T4 [Koffret](https://koffret.com/learn/mac-login-screen) |
| MC.07.01.x08 | behavior | Control Center | Full Keyboard Access can open Control Center with Fn-C. | T4 [Lets Suite](https://www.lets-suite.com/blog/full-keyboard-access-mac) |
| MC.07.01.x09 | changed | Control Center | The post reports smoother navigation in Control Center and Control Gallery, and a new option to turn off Control Gallery suggestions. | T5 [MacRumors Forums](https://forums.macrumors.com/threads/macos-27-all-the-little-things.2483520/) |
| MC.07.01.x10 | behavior | other | The article says the “Mickey Mouse hand” is back in macOS 27 Golden Gate. | T4 [Gizmodo](https://gizmodo.com/im-more-excited-for-the-tiny-changes-in-apples-27-software-updates-than-siri-ai-tbh-2000769815) |
| MC.07.01.x11 | behavior | Control Center | Do Not Disturb can also be turned on through Control Center > Focus > Do Not Disturb. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |

### MC.07.02 Third-party controls

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).

Also covered by: [Use Control Center on Mac](https://support.apple.com/guide/mac-help/mchl50f94f8f/27/mac/27)


#### Quickly change settings with Control Center on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlc9d0e1f2/27/mac/27) · macOS 27 text present: no

Control Center provides quick access to common Mac settings and media controls. You can customize its controls, pin controls to the menu bar, and check which apps are using the microphone, camera, or system audio.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.07.02.e001 | Control Center | panel | Opened from its icon in the menu bar | A panel showing all the controls at the same time. |
| MC.07.02.e002 | Control Center icon | icon | Menu bar | Click to open Control Center. |
| MC.07.02.e003 | Slider | control | Control Center | Drag to adjust volume or brightness. |
| MC.07.02.e004 | Arrow | control | Next to an item in Control Center | Click to reveal more detailed options. |
| MC.07.02.e005 | Wi-Fi | control | Control Center by default | Connect to wireless networks, switch between them, and see the current connection status. |
| MC.07.02.e006 | Bluetooth | control | Control Center by default | Turn Bluetooth on or off and see which devices are paired or connected. |
| MC.07.02.e007 | AirDrop | control | Control Center by default | Share files wirelessly with nearby iPhone, iPad, and Mac devices without setup. |
| MC.07.02.e008 | Focus | control | Control Center by default | Turn on Do Not Disturb or switch between Focus modes. |
| MC.07.02.e009 | Stage Manager | control | Control Center by default | Toggle Stage Manager to organize open windows into a focused workspace, with other apps tucked to the side. |
| MC.07.02.e010 | Screen Mirroring | control | Control Center by default | Mirror or extend the Mac display to an Apple TV or AirPlay-compatible screen. |
| MC.07.02.e011 | Display | control | Control Center by default | Adjust screen brightness and quickly turn Night Shift or True Tone on or off. |
| MC.07.02.e012 | Sound | control | Control Center by default | Adjust volume and switch the audio output device. |
| MC.07.02.e013 | Now Playing | control | Control Center by default | Control media playback from any app. |
| MC.07.02.e014 | Edit Controls | menu item | Bottom of the Control Center panel | Opens the controls customization interface. |
| MC.07.02.e015 | Battery | control | Available to add in Edit Controls | An optional control that can be added to Control Center. |
| MC.07.02.e016 | Accessibility Shortcuts | control | Available to add in Edit Controls | An optional control that can be added to Control Center. |
| MC.07.02.e017 | Fast User Switching | control | Available to add in Edit Controls | An optional control that can be added to Control Center. |
| MC.07.02.e018 | Done | button | Control customization interface | Finishes customization. |
| MC.07.02.e019 | Add to Menu Bar | menu item | Menu opened by Control-clicking a control | Pins the control to the menu bar. |
| MC.07.02.e020 | Privacy indicator dots | indicator | Menu bar | Part of Control Center; indicate use of the microphone, camera, or system audio. |
| MC.07.02.e021 | Notification Center | other | Mentioned in the Learn more section | Described as a place to get notifications from apps such as Messages and Mail. |
| MC.07.02.e022 | Spotlight | other | Mentioned in the Learn more section | Described as a way to search for anything on the Mac. |
| MC.07.02.e023 | Siri | other | Mentioned in the Learn more section | Described as a way to ask questions and take action. |
| MC.07.02.e024 | Safari | other | Mentioned near the end of the article | Identified as the built-in browser on Mac. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.07.02.b001 | Click the Control Center icon in the menu bar. | Control Center opens and shows all its controls at the same time. | None stated. |
| MC.07.02.b002 | Drag a slider in Control Center. | Adjusts volume or brightness. | The slider adjusts the corresponding setting. |
| MC.07.02.b003 | Click an icon in Control Center. | Toggles the associated feature on or off. | For controls that toggle a feature. |
| MC.07.02.b004 | Click the arrow next to an item. | Reveals more detailed options for that item. | When an arrow is shown next to the item. |
| MC.07.02.b005 | Use Wi-Fi in Control Center. | Connects to wireless networks, switches between them, and shows the current connection status. | Wi-Fi is a default Control Center control. |
| MC.07.02.b006 | Use Bluetooth in Control Center. | Turns Bluetooth on or off and shows paired or connected devices. | Bluetooth is a default Control Center control. |
| MC.07.02.b007 | Use AirDrop in Control Center. | Shares files wirelessly with nearby iPhone, iPad, and Mac devices. | The article says no setup is needed. |
| MC.07.02.b008 | Use Focus in Control Center. | Turns on Do Not Disturb or switches between Focus modes to manage interruptions. | Focus is a default Control Center control. |
| MC.07.02.b009 | Toggle Stage Manager in Control Center. | Organizes open windows into a focused workspace, with other apps tucked to the side. | Stage Manager is a default Control Center control. |
| MC.07.02.b010 | Use Screen Mirroring in Control Center. | Mirrors or extends the Mac display to an Apple TV or AirPlay-compatible screen. | Screen Mirroring is a default Control Center control. |
| MC.07.02.b011 | Use Display in Control Center. | Adjusts screen brightness and turns Night Shift or True Tone on or off. | Display is a default Control Center control. |
| MC.07.02.b012 | Use Sound in Control Center. | Adjusts volume and switches the audio output device. | Sound is a default Control Center control. |
| MC.07.02.b013 | Use Now Playing in Control Center. | Pauses, skips, or scrubs media playback without switching windows. | Controls media playback from any app. |
| MC.07.02.b014 | Click the Control Center icon and choose Edit Controls at the bottom of the panel. | Opens the interface for customizing Control Center. | None stated. |
| MC.07.02.b015 | Drag controls in or out in the customization interface. | Adds controls to or removes controls from Control Center. | While customizing Control Center. |
| MC.07.02.b016 | Change the order of controls in the customization interface. | Rearranges the controls. | While customizing Control Center. |
| MC.07.02.b017 | Control-click certain items in the customization interface. | Resizes those items. | Only certain items can be resized. |
| MC.07.02.b018 | Add Battery, Accessibility Shortcuts, or Fast User Switching in the customization interface. | Adds the selected control so it is one click away. | These controls do not appear by default. |
| MC.07.02.b019 | Click Done. | Finishes customizing Control Center. | After customization. |
| MC.07.02.b020 | Control-click a control and choose Add to Menu Bar. | Pins the control to the menu bar. | For a control that can be added to the menu bar. |
| MC.07.02.b021 | Hold Command and drag a menu bar icon off the bar. | Removes the icon from the menu bar. | None stated. |
| MC.07.02.b022 | Click a privacy indicator dot in the menu bar. | Shows which app is currently using the microphone, camera, or system audio. | When an indicator dot is present. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.07.02.k001 | Control-click | Resize certain controls while customizing, or open a control's menu for options such as Add to Menu Bar. | Control Center customization and controls. |
| MC.07.02.k002 | Command-drag | Drag a menu bar icon off the bar to remove it. | Menu bar. |

#### Controls

T1 · HIG (developer guidance) · [Apple HIG](https://developer.apple.com/design/human-interface-guidelines/controls) · macOS 27 text present: no

Controls provide quick access to app features from Control Center, the Lock Screen, or the Action button. The page states there are no additional considerations for macOS; it does not specify macOS-specific behavior.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.07.02.e025 | Control | control | Control Center, Lock Screen, or Action button | A button or toggle that provides quick access to an app feature. It can perform an action, link to an area of the app, or launch a camera experience on a locked device. |
| MC.07.02.e026 | Control Center | view | System interface | People can add controls by pressing and holding in an empty area. Controls show a symbol, and at larger sizes, a title and value. The controls gallery is available here. |
| MC.07.02.e027 | Lock Screen | view | System interface | People can add controls by customizing the Lock Screen. A control displays its symbol here. The controls gallery is available here. |
| MC.07.02.e028 | Action button | button | On supported iPhone devices | People configure a control for this button in the Settings app. Pressing and holding performs the configured action; the control’s symbol and, if present, value appear in the Dynamic Island. |
| MC.07.02.e029 | Settings app | other | System app | Used to configure a control for the Action button. |
| MC.07.02.e030 | Dynamic Island | view | On iPhone devices with a control assigned to the Action button | Displays the control’s symbol and, if present, its value when the Action button is pressed and held. |
| MC.07.02.e031 | controls gallery | view | Control Center or Lock Screen | Displays placeholder information when someone chooses a control, or before assigning it to the Action button. |
| MC.07.02.e032 | symbol image | icon | Within a control | Visually represents what the control does and may use an SF Symbol or a custom symbol. |
| MC.07.02.e033 | title | other | Within a control | Describes what the control relates to; it may be hidden in some control placements or redacted when the device is locked. |
| MC.07.02.e034 | value | other | Within a control | Represents the control’s state; it may be hidden in some control placements or redacted when the device is locked. |
| MC.07.02.e035 | camera experience | view | App camera launched from a control on a locked device | An app’s camera experience can launch directly from a control. Tasks beyond capture require authentication and unlocking. |

| ID | Context | Guideline | Conditions |
|---|---|---|---|
| MC.07.02.b023 | A person presses and holds in an empty area of Control Center. | They can add controls to Control Center. | As described on the page. |
| MC.07.02.b024 | A person customizes their Lock Screen. | They can add controls to the Lock Screen. | As described on the page. |
| MC.07.02.b025 | A person configures the Action button in the Settings app. | They can assign a control to the Action button. | On iPhone devices with an Action button. |
| MC.07.02.b026 | A control appears in Control Center. | It displays its symbol and, at larger sizes, its title and value. | The displayed content depends on control size. |
| MC.07.02.b027 | A control appears on the Lock Screen. | It displays its symbol. | As described on the page. |
| MC.07.02.b028 | A person presses and holds the Action button assigned to a control. | The system performs the configured action and displays the control’s symbol in the Dynamic Island, along with its value if present. | On iPhone devices with a control assigned to the Action button. |
| MC.07.02.b029 | Someone interacts with a control, an action completes, or an update arrives through a push notification. | Update the control so its contents accurately reflect the state and whether an action is still in progress. | Guideline. |
| MC.07.02.b030 | A control toggle changes between on and off. | Use symbols that suggest the behavior in both states, and animate the transition between the states. | Guideline. |
| MC.07.02.b031 | A control button performs an action with a duration. | Animate indefinitely while the action is performing, and stop animating when it completes. | Guideline. |
| MC.07.02.b032 | A control toggle is in its on state. | The system applies the selected tint color to its symbol. | Guideline. |
| MC.07.02.b033 | Someone performs a control’s action from the Action button. | The system uses the selected tint color to display the value and symbol in the Dynamic Island. | When applicable. |
| MC.07.02.b034 | A control needs additional information to perform its action. | Prompt people to provide the needed configuration when they first add the control; allow them to reconfigure it later. | For example, selecting a specific light. |
| MC.07.02.b035 | A person presses the Action button. | The system displays hint text describing what happens when they press and hold; use verbs in the hint text. | For controls assigned to the Action button. |
| MC.07.02.b036 | A control’s title or value can vary. | Include a placeholder describing the control when the title and value are situational. | The system displays it in the controls gallery or before Action button assignment. |
| MC.07.02.b037 | The device is locked and control information is sensitive. | Consider redacting the title and value; specify whether to redact the symbol state as well. If symbol-state redaction is specified, the system displays the symbol in its off state. | Guideline. |
| MC.07.02.b038 | A control action affects security. | Require authentication, such as unlocking the device, before the action can be accessed. | Examples include locking or unlocking a house door or starting a car. |
| MC.07.02.b039 | A control launches an app’s camera experience while the device is locked. | Launch directly into the camera experience; require authentication and unlocking for tasks beyond capture. | Starting with iOS 18; the page gives no macOS-specific camera guidance. |
| MC.07.02.b040 | Someone captures content in the camera experience and taps a button for additional tasks. | Use the same camera UI in the app and camera experience to make the transition seamless. | Guideline. |
| MC.07.02.b041 | An app provides a control that launches a camera experience. | Provide instructions for adding the control. | Guideline. |

API names: `promptsForUserConfiguration()`, `controlWidgetActionHint(_:)`, `IntentAuthenticationPolicy`, `LockedCameraCapture`, `WidgetKit`, `SymbolEffect`, `SF Symbols`

System Settings paths referenced: Settings app

### MC.07.03 Notification grouping

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).


#### View app notifications on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mh40609/27/mac/27) · macOS 27 text present: no

Explains how to view, expand, act on, and clear app notifications on Mac, and how notification choices relate to Focus.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.07.03.e001 | Show less | button | On an expanded stack of notifications | Collapses the stack. |
| MC.07.03.e002 | Read Story | button | In the example News notification | An example action available in a notification. |
| MC.07.03.e003 | Options | button | In a notification | Opens notification options; the example uses it to choose a Snooze duration. |
| MC.07.03.e004 | Snooze duration | menu item | In the Calendar notification example, after choosing Options | A duration choice for snoozing. |
| MC.07.03.e005 | Notifications settings | setting | Referenced in the article | Stores notification choices and can be changed at any time. |
| MC.07.03.e006 | Focus | setting | Referenced in the article | Can be used to silence all notifications or allow only certain notifications. |
| MC.07.03.e007 | Do Not Disturb | setting | Listed as an example of a Focus | An example Focus. |
| MC.07.03.e008 | Work | setting | Listed as an example of a Focus | An example Focus. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.07.03.b001 | An app notification arrives | It appears briefly in the top-right corner of the desktop or stays there until closed. | Applies to notifications described in the article. |
| MC.07.03.b002 | Move the pointer over a notification | The notification's available controls can be used. | The article introduces the listed notification actions with this trigger. |
| MC.07.03.b003 | Click anywhere in the top notification | Expands the stack to show all notifications. | The app's notifications are grouped and stacked. |
| MC.07.03.b004 | Click “Show less” | Collapses the notification stack. | The stack is expanded. |
| MC.07.03.b005 | Click an action in a notification | Performs that action; the example is clicking “Read Story” in a News notification. | An action is available in the notification. |
| MC.07.03.b006 | Click “Options,” then choose a Snooze duration | Chooses a snooze duration. | The example is a Calendar notification. |
| MC.07.03.b007 | Click a notification | Opens the item in the app. | The article describes this as a way to see more details. |
| MC.07.03.b008 | Mute notifications | Notifications are not shown and do not play a sound when they arrive. | The article's preceding control description is incomplete. |
| MC.07.03.b009 | Clear a single notification | The notification is cleared. | The article says to click a control, but its icon or label is missing from the supplied text. |
| MC.07.03.b010 | Clear all notifications in a stack | All notifications in the stack are cleared. | The supplied instruction is cut off after “Hold the pointer over the”; the remaining control or target is not provided. |
| MC.07.03.b011 | Open an app for the first time | The user is asked whether to receive notifications from that app. | Applies to some apps. |
| MC.07.03.b012 | Receive a time-sensitive or critical notification for the first time | The user is asked whether to allow that type of notification when a Focus is active. | Time-sensitive notifications are exemplified by Calendar or Reminders; critical notifications are exemplified by Home. |
| MC.07.03.b013 | Change notification choices | The choices are saved in Notifications settings and can be changed at any time. | Applies to the notification choices described in the article. |
| MC.07.03.b014 | Use a Focus | Silences all notifications or allows only certain notifications to appear. | Do Not Disturb and Work are given as examples. |

System Settings paths referenced: Notifications settings

#### Get notifications on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchle7f8a9b0/27/mac/27) · macOS 27 text present: no

Notifications appear as brief banners in the top-right corner of the screen. Notification Center lets you review and act on notifications and manage widgets.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.07.03.e009 | Notification Center | view | Opened from the top-right corner of the menu bar | Displays notifications and, below them, widgets. |
| MC.07.03.e010 | date and time | control | Top-right corner of the menu bar | Click to open Notification Center. |
| MC.07.03.e011 | menu bar | view | Top of the screen | The date and time at its top-right corner can be clicked to open Notification Center. |
| MC.07.03.e012 | notification | view | Top-right corner of the screen, and in Notification Center | A brief alert from an app; hovering over one in Notification Center reveals options. |
| MC.07.03.e013 | Reply | menu item | Inline actions on a notification | An example of an action that can respond to a notification without leaving what you’re doing. |
| MC.07.03.e014 | Snooze | menu item | Inline actions on a notification | An example of an action available on a notification. |
| MC.07.03.e015 | Archive | menu item | Inline actions on a notification | An example of an action available on a notification. |
| MC.07.03.e016 | stack | view | Notification Center | Notifications from the same app are grouped together. |
| MC.07.03.e017 | Show Less | button | Expanded notification stack | Collapses the stack. |
| MC.07.03.e018 | widgets | view | Below notifications in Notification Center | Live snippets from apps, including Calendar, Weather, and Stocks. |
| MC.07.03.e019 | Calendar | other | A widget example in Notification Center | Named as an example of an app that provides a widget. |
| MC.07.03.e020 | Weather | other | A widget example in Notification Center | Named as an example of an app that provides a widget. |
| MC.07.03.e021 | Stocks | other | A widget example in Notification Center | Named as an example of an app that provides a widget. |
| MC.07.03.e022 | size menu | menu | Shown after Control-clicking a widget | Offers a size to choose for the widget; the size options are not specified. |
| MC.07.03.e023 | remove button (–) | button | On a widget after holding Option and hovering over it | Removes the widget. |
| MC.07.03.e024 | Focus | setting | Mentioned in the tip about silencing notifications | Can be used to silence notifications when you need to concentrate. |
| MC.07.03.e025 | Control Center | view | Mentioned in the Focus tip and the related-learning section | The article says it can be used to quickly change settings such as screen brightness, connect to Wi-Fi, and turn on Do Not Disturb. |
| MC.07.03.e026 | Do Not Disturb | setting | Control Center | Named as a setting that can be turned on from Control Center. |
| MC.07.03.e027 | Siri | other | Related-learning section | The article says you can ask Siri questions and take action. |
| MC.07.03.e028 | Spotlight | other | Related-learning section | Named as a way to search for anything on your Mac. |
| MC.07.03.e029 | Explore notifications | other | Related content beside the Act on notifications section | Related-content link text. |
| MC.07.03.e030 | Learn more | other | Related-learning section | Link text shown for Control Center and Spotlight. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.07.03.b015 | Click the date and time in the top-right corner of the menu bar. | Opens Notification Center. | — |
| MC.07.03.b016 | Swipe left from the right edge of the trackpad with two fingers. | Opens Notification Center. | Requires a trackpad. |
| MC.07.03.b017 | Click anywhere on the desktop. | Closes Notification Center. | Notification Center is open. |
| MC.07.03.b018 | Swipe right. | Closes Notification Center. | Notification Center is open. |
| MC.07.03.b019 | A message, reminder, or app alert arrives. | A brief notification banner appears in the top-right corner of the screen and disappears on its own after a moment. | — |
| MC.07.03.b020 | Open Notification Center. | Review notifications you missed. | — |
| MC.07.03.b021 | Hold the pointer over a notification in Notification Center. | Options for the notification appear. | — |
| MC.07.03.b022 | Click a notification. | Opens the relevant app. | The notification is in Notification Center. |
| MC.07.03.b023 | Use an inline action, such as Reply, Snooze, or Archive. | Responds to the notification without leaving what you’re doing. | The notification offers the relevant inline action. |
| MC.07.03.b024 | Click a notification stack. | Expands the stack. | The stack contains notifications from the same app. |
| MC.07.03.b025 | Read through the items in an expanded stack, then click Show Less. | Collapses the stack. | — |
| MC.07.03.b026 | Click a widget. | Jumps to the widget’s full details in the app. | — |
| MC.07.03.b027 | Control-click a widget and choose a size from the menu. | Resizes the widget. | The size options are not specified. |
| MC.07.03.b028 | Hold Option, hover over a widget, then click the remove button (–) that appears. | Removes the widget. | — |
| MC.07.03.b029 | Use Focus. | Silences notifications. | When you need to concentrate. |

#### Use Notification Center on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchl2fb1258f/27/mac/27) · macOS 27 text present: no

Notification Center lets you review notifications and use widgets on the desktop. You can open or close it, interact with notifications and widgets, and use a Focus to reduce distractions.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.07.03.e031 | Notification Center | panel | On the Mac desktop | Shows notifications and widgets. |
| MC.07.03.e032 | date and time | control | Menu bar | Click to open or close Notification Center. |
| MC.07.03.e033 | menu bar | view | Top of the screen | Contains the date and time control mentioned in the instructions. |
| MC.07.03.e034 | desktop | view | Mac desktop | Click anywhere on it to close Notification Center. |
| MC.07.03.e035 | notification | view | Notification Center | Can be opened to view its item in the associated app. |
| MC.07.03.e036 | Show less | button | Expanded stack of notifications | Collapses a stack of grouped notifications. |
| MC.07.03.e037 | Snooze | button | Calendar notification | Example of an action available in a notification. |
| MC.07.03.e038 | Reply | button | Mail notification | Example of an action available in a notification. |
| MC.07.03.e039 | arrow | indicator | An action or notification | The source mentions an arrow as a condition for additional options, but the corresponding instruction is incomplete. |
| MC.07.03.e040 | Notification Settings | menu item | Options associated with a notification | Choose it to change an app’s notification settings. |
| MC.07.03.e041 | widget | view | Notification Center | Can show information and open a related setting, app, or webpage when clicked. |
| MC.07.03.e042 | Reminders | other | Reminders widget | The example says clicking the widget opens the Reminders app. |
| MC.07.03.e043 | Weather | other | Weather widget | The example says clicking the widget opens the Weather app and its complete forecast. |
| MC.07.03.e044 | Option | other | Keyboard | Press and hold while moving the pointer over a widget to reveal its remove control. |
| MC.07.03.e045 | Focus | setting | Mac | Can be used to silence all notifications or allow only certain notifications. |
| MC.07.03.e046 | Do Not Disturb | setting | Focus | An example of a Focus. |
| MC.07.03.e047 | Work | setting | Focus | An example of a Focus. |
| MC.07.03.e048 | Add and customize widgets | other | Linked guide | Referenced for changing and customizing widgets shown in Notification Center. |
| MC.07.03.e049 | Set up a Focus | other | Linked guide | Referenced for using a Focus. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.07.03.b030 | Click the date and time in the menu bar. | Opens Notification Center. | On a Mac. |
| MC.07.03.b031 | Swipe left with two fingers from the right edge of the trackpad. | Opens Notification Center. | When using a trackpad. |
| MC.07.03.b032 | Click anywhere on the desktop. | Closes Notification Center. | On a Mac. |
| MC.07.03.b033 | Click the date and time in the menu bar. | Closes Notification Center. | When Notification Center is open. |
| MC.07.03.b034 | Swipe right with two fingers toward the right edge of the trackpad. | Closes Notification Center. | When Notification Center is open and using a trackpad. |
| MC.07.03.b035 | Move the pointer over a notification. | Makes the notification available for the described interactions. | In Notification Center. |
| MC.07.03.b036 | Click anywhere in the top notification. | Expands a stack and shows all notifications in it. | The app’s notifications are grouped in a stack. |
| MC.07.03.b037 | Click “Show less.” | Collapses the notification stack. | The stack is expanded. |
| MC.07.03.b038 | Click an action in a notification, such as Snooze or Reply. | Performs that notification action. | The notification provides the action; Snooze is given as a Calendar example and Reply as a Mail example. |
| MC.07.03.b039 | Click a notification. | Opens the item in the app. | In Notification Center. |
| MC.07.03.b040 | Choose to mute or turn off notifications, or choose Notification Settings. | Mutes or turns off notifications, or opens notification settings to change them. | The source introduces this with “If an arrow” but the condition is incomplete. |
| MC.07.03.b041 | Click the control for clearing a single notification. | Clears that notification. | The source’s control icon is missing. |
| MC.07.03.b042 | Click anywhere in a widget. | Opens the related setting, app, or webpage. | In Notification Center. The examples are clicking the Reminders widget to open Reminders, or the Weather widget to open Weather and view the complete forecast. |
| MC.07.03.b043 | Control-click a widget and choose a different size. | Resizes the widget. | In Notification Center. |
| MC.07.03.b044 | Press and hold Option while moving the pointer over a widget, then click the remove control. | Removes the widget. | In Notification Center; the source’s remove-control icon is missing. |
| MC.07.03.b045 | Use a Focus, such as Do Not Disturb or Work. | Silences all notifications or allows only certain notifications to appear. | Suggested when you need to minimize distractions. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.07.03.k001 | Two-finger swipe left | Open Notification Center. | Swipe from the right edge of the trackpad. |
| MC.07.03.k002 | Two-finger swipe right | Close Notification Center. | Swipe toward the right edge of the trackpad. |
| MC.07.03.k003 | Control-click | Open the widget’s contextual options, then choose a different size. | On a widget in Notification Center. |
| MC.07.03.k004 | Option | Reveal the widget’s remove control while moving the pointer over it; click that control to remove the widget. | Press and hold Option while moving the pointer over a widget. |

#### Notifications

T1 · HIG (developer guidance) · [Apple HIG](https://developer.apple.com/design/human-interface-guidelines/notifications) · macOS 27 text present: no

Apple’s notification guidance applies across platforms, with no additional macOS-specific considerations. It covers notification content, actions, sounds, badges, and handling notifications while an app is in the foreground.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.07.03.e050 | desktop | view | Notification anatomy | One possible place a notification can appear. |
| MC.07.03.e051 | Notification Center | view | Notification anatomy | A place where notifications can appear. |
| MC.07.03.e052 | banner | view | Notification anatomy | One possible notification style, depending on the platform. |
| MC.07.03.e053 | badge | indicator | App icon | A small, filled oval containing a number that can indicate unread notifications. |
| MC.07.03.e054 | title | other | Top of a notification | Displays at the top when provided; otherwise, a noncommunication notification displays the app name. |
| MC.07.03.e055 | Notification | other | Notification with previews hidden | The default title shown with the app icon when notification previews aren’t available. |
| MC.07.03.e056 | notification actions | view | Customizable notification detail view | Can contain up to four buttons for actions people can perform without opening the app. |
| MC.07.03.e057 | Snooze | button | Calendar event notification example | Example action that postpones an event’s alarm. |
| MC.07.03.e058 | Settings | other | Notification preview preferences | People can choose to hide notification previews for all apps. |

| ID | Context | Guideline | Conditions |
|---|---|---|---|
| MC.07.03.b046 | Before an app sends notifications | Get the person’s consent. | Required before sending any notifications. |
| MC.07.03.b047 | When composing a notification | Make it concise and informative, and avoid sending multiple notifications for the same thing. | People use notifications for quick updates; repeated notifications can fill Notification Center. |
| MC.07.03.b048 | When a notification suggests an in-app task | Avoid telling people to perform a specific task in the app; offer a notification action if the task is simple and can be done without opening the app. | Otherwise, avoid instructions that may be forgotten after the notification is dismissed. |
| MC.07.03.b049 | When displaying an error message | Use an alert, not a notification. | — |
| MC.07.03.b050 | When the app is in the foreground and receives notification information | Present the information in a discoverable but nondistracting way, such as updating a badge or subtly inserting data into the current view. | The app’s notifications don’t appear while the app is in front, but the app still receives the information. |
| MC.07.03.b051 | When notification content could be visible to other people | Avoid including sensitive, personal, or confidential information. | The developer can’t predict what the person will be doing when the notification arrives. |
| MC.07.03.b052 | When providing a title | Use a brief, useful title when it adds context; use title-style capitalization and no ending punctuation. | The title appears prominently. If a noncommunication notification would have only a generic title, it may be better to let the system display the app name. |
| MC.07.03.b053 | When writing notification content | Use complete sentences, sentence case, and proper punctuation; don’t truncate the message. | The system truncates content when necessary. |
| MC.07.03.b054 | When notification previews may be hidden | Provide generically descriptive body text that gives context without revealing too many details. | When previews are unavailable, the system shows the app icon and the default title “Notification.” |
| MC.07.03.b055 | When configuring a notification | Don’t include the app name or icon in the content. | The system displays the app icon; for communication notifications, it displays the sender’s contact image with a small app icon. |
| MC.07.03.b056 | When choosing whether to include notification sound | A sound may supplement a notification, but don’t rely on it to communicate important information. | A custom sound should be short, distinctive, and professionally produced. Vibration can’t be provided programmatically. |
| MC.07.03.b057 | When adding notification action buttons | Choose beneficial, context-appropriate actions that save time; use brief, clear, title-case labels and a recognizable interface icon for each action. | A detail view can contain up to four buttons. Account for localization and avoid labels that include the app name or extraneous information. |
| MC.07.03.b058 | When deciding whether to add an action that opens the app | Avoid an action that merely opens the app. | Tapping the notification or its preview is expected to display related content. |
| MC.07.03.b059 | When an action could be destructive | Prefer nondestructive actions; provide enough context if a destructive action is necessary. | The system gives actions identified as destructive a distinct appearance. |
| MC.07.03.b060 | When displaying an app badge | Use it only to show the number of unread notifications, keep it up to date, and don’t make it the only way to communicate essential information. | People can disable badges. Reducing the count to zero removes all related notifications from Notification Center. |
| MC.07.03.b061 | When designing a notification indicator | Don’t create a custom image or component that mimics a badge. | People may have turned off notification badges. |

API names: `hiddenBodyPlaceholder`, `UNNotificationSound`

System Settings paths referenced: Settings (notification preview visibility); notification settings (badge permission)

#### Independent reports (MC.07.03)

| ID | Kind | Area | Claim | Source |
|---|---|---|---|---|
| MC.07.03.x01 | bug | notifications | The reporter said clicking the Notification Center button at the top right did nothing while menu icons were hidden, preventing access to Notification Center. | T2 [GitHub jacklandrin/OnlySwitch](https://github.com/jacklandrin/OnlySwitch/issues/218) |
| MC.07.03.x02 | behavior | notifications | The pull request reports that Notification Center opens with Globe+N when no assessment-mode assertion is held, but the shortcut does nothing while one is held. | T2 [GitHub jordanbaird/Ice](https://github.com/jordanbaird/Ice/pull/995) |
| MC.07.03.x03 | behavior | notifications | The article says an automation sends a notification asking the user to run it when its event occurs; automations sync to other devices disabled. | T4 [Pizazoo](https://pizazoo.com/blog/macos-27-screenshot-features/) |
| MC.07.03.x04 | behavior | notifications | Full Keyboard Access can open Notification Center with Fn-N. | T4 [Lets Suite](https://www.lets-suite.com/blog/full-keyboard-access-mac) |
| MC.07.03.x05 | bug | notifications | Fixed an issue where certain notifications could not be dismissed. | T4 [BetaProfile](https://betaprofile.dev/macos-27-rc/) |
| MC.07.03.x06 | changed | notifications | In macOS Big Sur and newer, the dedicated Notification Center menu bar icon was removed. Select “Date & Time” in the top-right corner of the display to open Notification Center; select it again or click outside to close it. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |
| MC.07.03.x07 | behavior | notifications | The article suggests setting Notification Center as a hot corner for quick access. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |
| MC.07.03.x08 | behavior | notifications | In macOS Catalina and Mojave, Notification Center has separate “Today” and “Notifications” tabs. Widgets appear in Today; new and missed notifications appear in Notifications. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |
| MC.07.03.x09 | changed | notifications | In macOS Big Sur and newer, the Today and Notifications tabs are merged, with notifications at the top and widgets below. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |
| MC.07.03.x10 | behavior | notifications | Selecting a notification opens its content in the associated app. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |
| MC.07.03.x11 | behavior | notifications | Hover over a notification and select the “Arrow icon” in its upper-right corner to reveal additional options. The available options depend on the app and notification type. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |
| MC.07.03.x12 | behavior | notifications | Some notifications are grouped or stacked by app or type. Select a stack or group to expand it; select the “X-shaped icon” in the top corner to dismiss a notification or stack. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |
| MC.07.03.x13 | behavior | notifications | A Messages notification can provide an option to compose and send a reply from Notification Center. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |
| MC.07.03.x14 | behavior | notifications | Right-click an app notification and select “Deliver Quietly” to have future notifications from that app go directly to Notification Center rather than appear on the desktop. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |
| MC.07.03.x15 | behavior | notifications | Notifications from an app can also be muted temporarily for one hour or for the entire day. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |
| MC.07.03.x16 | behavior | notifications | To unmute an app, right-click one of its notifications in Notification Center and select “Unmute” or “Deliver Prominently”. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |
| MC.07.03.x17 | behavior | notifications | Do Not Disturb stays on until turned off, or can be enabled temporarily using “For 1 hour” or “Until this evening”. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |
| MC.07.03.x18 | behavior | notifications | A Do Not Disturb schedule can be configured at System Settings > Focus > Do Not Disturb > Add Schedule, with schedule types “Time”, “Location”, or “App”. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |
| MC.07.03.x19 | behavior | notifications | Right-click an app notification and select “Turn Off” to stop receiving notifications from that app. To enable them again, go to System Settings > Notifications, select the app, and turn on “Allow Notifications”. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |
| MC.07.03.x20 | behavior | notifications | Notification grouping is configured per app in System Settings > Notifications, using the “Notification grouping” menu. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |
| MC.07.03.x21 | behavior | notifications | The “Automatic” grouping option stacks notifications by app or website; “By Application” groups by app and lists the most recent notification first; “Off” disables grouping for that app. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |

### MC.07.04 Widgets on desktop

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).


#### Add and customize widgets on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchl52be5da5/27/mac/27) · macOS 27 text present: no

Instructions for adding widgets to the desktop or Notification Center, using iPhone widgets, customizing or removing widgets, and changing widget settings. No macOS version or macOS 27 changes are stated.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.07.04.e001 | Notification Center | panel | Opened from the menu bar | Shows widgets; open it by clicking the date and time or swiping left with two fingers from the right edge of the trackpad. |
| MC.07.04.e002 | date and time | other | Menu bar | Click to open Notification Center. |
| MC.07.04.e003 | menu bar | other | Top of the screen | Contains the date and time used to open Notification Center. |
| MC.07.04.e004 | trackpad | other | Right edge of the trackpad | Swipe left with two fingers from its right edge to open Notification Center. |
| MC.07.04.e005 | wallpaper | other | Desktop | Control-click it to choose Edit Widgets; clicking it can move open windows away. |
| MC.07.04.e006 | Edit Widgets | menu item | Wallpaper shortcut menu or bottom of Notification Center | Opens the widget gallery for adding widgets; also available from the bottom of Notification Center. |
| MC.07.04.e007 | Click wallpaper to reveal desktop | setting | System Settings > Desktop & Dock | Controls whether clicking the desktop moves open windows away. |
| MC.07.04.e008 | Only in Stage Manager | other | Click wallpaper to reveal desktop setting | When selected, clicking the desktop moves windows away only when using Stage Manager. |
| MC.07.04.e009 | Stage Manager | other | Desktop & Dock settings | A mode referenced by the wallpaper-click setting and the Show Widgets option. |
| MC.07.04.e010 | widget gallery | view | Opened with Edit Widgets | Search for widgets or browse categories; click or drag widgets to add them. |
| MC.07.04.e011 | Add button | button | On a widget in the widget gallery | Adds the widget. |
| MC.07.04.e012 | Remove button | button | On a newly added widget or in the widget gallery | Removes a widget in the described add-widget flows. |
| MC.07.04.e013 | Done | button | Bottom-right corner of the widget gallery | Finishes adding widgets. |
| MC.07.04.e014 | System Settings | other | Mac | Open this app to change widget settings. |
| MC.07.04.e015 | Desktop & Dock | other | System Settings | Contains the Widgets settings. |
| MC.07.04.e016 | Widgets | panel | System Settings > Desktop & Dock | Contains widget display, dimming, and iPhone-widget settings. |
| MC.07.04.e017 | Use iPhone widgets | setting | System Settings > Desktop & Dock > Widgets | Turn on to make eligible iPhone widgets available in the widget gallery. |
| MC.07.04.e018 | Open [app name] on your iPhone to continue | other | Message shown when clicking certain iPhone widgets | Indicates the app isn’t installed on the Mac and must be opened on the iPhone to interact with it. |
| MC.07.04.e019 | Edit [widget name] | menu item | Widget shortcut menu | Opens options for changing the information shown, when available. |
| MC.07.04.e020 | List widget for Reminders | other | Widget example in customization instructions | Its highlighted list can be clicked to choose a different reminder list. |
| MC.07.04.e021 | Done | button | In an edited widget | Finishes changing widget information. |
| MC.07.04.e022 | Remove Widget | menu item | Widget shortcut menu | Removes the selected widget. |
| MC.07.04.e023 | Show Widgets: | other | System Settings > Desktop & Dock > Widgets | Checkbox options control whether widgets are shown on the desktop and in Stage Manager. |
| MC.07.04.e024 | On Desktop: | setting | Show Widgets: in Widgets settings | Show widgets on the desktop. |
| MC.07.04.e025 | In Stage Manager: | setting | Show Widgets: in Widgets settings | Show widgets when Stage Manager is turned on. |
| MC.07.04.e026 | Dim widgets on desktop | control | System Settings > Desktop & Dock > Widgets | Pop-up menu for choosing widget color appearance. |
| MC.07.04.e027 | Automatically | menu item | Dim widgets on desktop pop-up menu | Automatically switches between monochrome and full-color. |
| MC.07.04.e028 | Always | menu item | Dim widgets on desktop pop-up menu | Always shows widgets in monochrome. |
| MC.07.04.e029 | Never | menu item | Dim widgets on desktop pop-up menu | Always shows widgets in full-color. |
| MC.07.04.e030 | iPhone Widgets | setting | System Settings > Desktop & Dock > Widgets | Turn iPhone widgets on or off. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.07.04.b001 | Click the date and time in the menu bar. | Opens Notification Center. | On a Mac. |
| MC.07.04.b002 | Swipe left with two fingers from the right edge of the trackpad. | Opens Notification Center. | On a Mac with a trackpad. |
| MC.07.04.b003 | Click anywhere on the desktop. | Closes Notification Center. | When Notification Center is open. |
| MC.07.04.b004 | Click the wallpaper. | Moves open windows away so desktop widgets can be seen. | When desktop widgets are hidden by other windows, subject to the Click wallpaper to reveal desktop setting. |
| MC.07.04.b005 | Set Click wallpaper to reveal desktop to Only in Stage Manager, then click the desktop. | Open windows move away only when Stage Manager is being used. | The setting is Only in Stage Manager. |
| MC.07.04.b006 | Control-click the wallpaper and choose Edit Widgets. | Opens the widget gallery. | To add widgets to the desktop. |
| MC.07.04.b007 | Search in the widget gallery or click a category. | Finds widgets or displays available widgets in that category. | When the widget gallery is open. |
| MC.07.04.b008 | Click a widget or its Add button in the gallery. | Automatically positions the widget on the desktop. | Adding a widget to the desktop. |
| MC.07.04.b009 | Drag a widget from the gallery to a desktop position. | Adds the widget at that position. | Adding a widget to the desktop. |
| MC.07.04.b010 | Drag a newly added desktop widget. | Moves it to another desktop location. | After adding the widget. |
| MC.07.04.b011 | Click the widget’s Remove button. | Removes the new widget. | When deciding not to keep a newly added desktop widget. |
| MC.07.04.b012 | Click Done at the bottom-right of the widget gallery. | Finishes adding widgets. | After adding widgets. |
| MC.07.04.b013 | Open Notification Center and click Edit Widgets at its bottom. | Opens the widget gallery. | To add widgets to Notification Center. |
| MC.07.04.b014 | Drag a widget to the upper-right corner of the desktop. | Adds it to Notification Center. | In the Add widgets to Notification Center flow. |
| MC.07.04.b015 | Click a widget or its Add button in the gallery. | Adds the widget to Notification Center. | In the Add widgets to Notification Center flow. |
| MC.07.04.b016 | Drag a widget up or down in Notification Center. | Changes its location in Notification Center. | After adding the widget. |
| MC.07.04.b017 | Click the widget’s Remove button. | Removes the widget. | When deciding not to keep a newly added Notification Center widget. |
| MC.07.04.b018 | Click Done at the bottom-right of the widget gallery. | Finishes adding widgets. | After adding widgets to Notification Center. |
| MC.07.04.b019 | Use an iPhone widget on the Mac. | Makes widgets from apps installed on the iPhone available in the Mac widget gallery. | The iPhone uses iOS 17 or later; the same Apple Account is signed in on iPhone and Mac; and the iPhone is near the Mac or on the same Wi-Fi network. |
| MC.07.04.b020 | In System Settings, click Desktop & Dock, then turn on Use iPhone widgets under Widgets. | Makes eligible iPhone widgets available to add to the desktop or Notification Center from the widget gallery. | The stated iPhone-widget requirements are met. |
| MC.07.04.b021 | Click an iPhone widget and see “Open [app name] on your iPhone to continue.” | Open the app on the iPhone to interact with the widget. | The app isn’t installed on the Mac. Some third-party apps aren’t available for both iPhone and Mac. |
| MC.07.04.b022 | Control-click a widget and choose Edit [widget name]. | Shows options to change the information displayed; click highlighted information to change it, then click Done in the widget. | Edit [widget name] is available in the shortcut menu. |
| MC.07.04.b023 | Look for Edit [widget name] in a widget’s shortcut menu and find it unavailable. | The widget’s displayed information can’t be changed. | The menu item isn’t available. |
| MC.07.04.b024 | Click the highlighted list in the List widget for Reminders. | Choose a different reminder list. | When editing that widget. |
| MC.07.04.b025 | Choose a different size from a widget’s shortcut menu. | Changes the widget’s size. | When customizing a widget. |
| MC.07.04.b026 | Choose Remove Widget from a widget’s shortcut menu. | Removes the widget. | When customizing a widget. |
| MC.07.04.b027 | Click the wallpaper, then choose Edit Widgets. | Opens the widget gallery; click the Remove button to remove a desktop widget. | To remove widgets from the desktop. |
| MC.07.04.b028 | Control-click a widget in Notification Center and choose Remove Widget. | Removes the widget. | To remove a widget from Notification Center. |
| MC.07.04.b029 | Move the pointer over a widget in Notification Center, press and hold Option, then click the remove control. | Removes the widget. | To remove a widget from Notification Center. |
| MC.07.04.b030 | Turn off On Desktop under Show Widgets:. | Widgets are not shown on the desktop. | In System Settings > Desktop & Dock > Widgets. |
| MC.07.04.b031 | Turn off In Stage Manager under Show Widgets:. | Items on the desktop are hidden; click the desktop to show the items when you want to access them. | When Stage Manager is turned on. |
| MC.07.04.b032 | Choose Automatically in the Dim widgets on desktop menu. | Widgets automatically switch between monochrome and full-color. | In Widgets settings. |
| MC.07.04.b033 | Choose Always in the Dim widgets on desktop menu. | Widgets are always shown in monochrome. | In Widgets settings. |
| MC.07.04.b034 | Choose Never in the Dim widgets on desktop menu. | Widgets are always shown in full-color. | In Widgets settings. |
| MC.07.04.b035 | Turn iPhone Widgets on or off. | Enables or disables iPhone widgets. | In System Settings > Desktop & Dock > Widgets. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.07.04.k001 | Two-finger swipe left from the right edge of the trackpad | Open Notification Center. | On a Mac. |
| MC.07.04.k002 | Control-click | Open the contextual menu; for the wallpaper, choose Edit Widgets; for a widget, access available widget options. | On the desktop wallpaper or a widget, as described. |
| MC.07.04.k003 | Press and hold Option, then click | Remove the widget under the pointer. | A widget in Notification Center; use the remove control. |

System Settings paths referenced: System Settings > Desktop & Dock > Click wallpaper to reveal desktop; System Settings > Desktop & Dock > Widgets > Use iPhone widgets; System Settings > Desktop & Dock > Widgets > Show Widgets: > On Desktop:; System Settings > Desktop & Dock > Widgets > Show Widgets: > In Stage Manager:; System Settings > Desktop & Dock > Widgets > Dim widgets on desktop; System Settings > Desktop & Dock > Widgets > iPhone Widgets

#### Widgets

T1 · HIG (developer guidance) · [Apple HIG](https://developer.apple.com/design/human-interface-guidelines/widgets) · macOS 27 text present: no

Widgets can appear on the Mac desktop and in Notification Center. The guidance describes supported widget sizes, rendering modes, margins, updates, and interactions. The page says there are no additional macOS-specific considerations.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.07.04.e031 | Desktop | view | Mac widget contexts | A location where system family widgets appear. The rendering-mode table lists full-color and vibrant appearances for widgets on the Mac desktop. |
| MC.07.04.e032 | Notification Center | view | Mac widget contexts | A location where system family widgets appear. |
| MC.07.04.e033 | System small | other | System family widgets | A system family widget size supported on Mac desktop and in Notification Center. |
| MC.07.04.e034 | System medium | other | System family widgets | A system family widget size supported on Mac desktop and in Notification Center. |
| MC.07.04.e035 | System large | other | System family widgets | A system family widget size supported on Mac desktop and in Notification Center. |
| MC.07.04.e036 | System extra large | other | System family widgets | A system family widget size supported on Mac desktop and in Notification Center. |
| MC.07.04.e037 | full-color | indicator | Mac widgets | A rendering mode that displays a widget in full color without changing the colors of its views. |
| MC.07.04.e038 | accented | indicator | Mac widgets | A rendering mode listed as not supported on Mac. |
| MC.07.04.e039 | vibrant | indicator | Mac desktop | A rendering mode listed for widgets on the Mac desktop. |

| ID | Context | Guideline | Conditions |
|---|---|---|---|
| MC.07.04.b036 | A person views a widget on a Mac. | The widget may appear on the desktop or in Notification Center. | System family widgets are supported in both contexts. |
| MC.07.04.b037 | A system family widget is displayed on a Mac. | The system uses the full-color rendering mode to display the widget in full color without changing the colors of its views. | The page says full-color rendering is used for system family widgets across all platforms. |
| MC.07.04.b038 | A widget is displayed on the Mac desktop. | The rendering-mode table lists full-color and vibrant rendering modes. | The table lists accented rendering as not supported on Mac. |
| MC.07.04.b039 | A widget is displayed on the Mac desktop. | Use smaller margins than the standard widget margins. | The page says widgets use smaller margins on the Mac desktop; it does not specify a Mac margin value. |
| MC.07.04.b040 | A widget’s content changes. | Use standard or custom animations, with a duration of up to two seconds, to draw attention to the update or changed display. | The page notes that many SwiftUI views animate content updates by default. |
| MC.07.04.b041 | A widget’s data changes over time. | Keep the widget up to date with an appropriate refresh frequency, and consider showing when its data was last updated if people may check it more often than it can refresh. | Widgets refresh periodically, do not support continuous real-time updates, and system update limits may vary. |
| MC.07.04.b042 | A person clicks a widget. | Launch the corresponding app and deep link to the content or action related to the widget. | The page recommends opening the app at the relevant location. |
| MC.07.04.b043 | A person clicks a widget button or toggle. | Offer functionality related to the widget’s content without launching the app. | Keep interactions simple and relevant; interactions outside buttons or toggles launch the app. |
| MC.07.04.b044 | Widget content is laid out near the widget’s edges. | Use standard margins for legibility, or tighter margins when needed for content groupings. | The general guidance gives 16 points for most widgets and says 11 points can work for tighter margins; Mac desktop widgets use smaller margins. |
| MC.07.04.b045 | A widget includes text. | Prefer system fonts, text styles, and SF Symbols; generally use text at 11 points or larger, and avoid rasterizing text. | The page presents this as general widget guidance. |
| MC.07.04.b046 | A widget is being designed for Mac. | Apply the general widget design guidance; there are no additional macOS considerations listed. | The Platform considerations section states: “No additional considerations for macOS.” |

API names: `WidgetKit`, `WidgetRendering`, `padding(_:_:)`, `ContainerRelative`, `widgetAccentable(_:)`

#### Independent reports (MC.07.04)

| ID | Kind | Area | Claim | Source |
|---|---|---|---|---|
| MC.07.04.x01 | changed | widgets | The weather widget in the menu bar has animation. | T3 [Wikipedia](https://en.wikipedia.org/wiki/MacOS_Golden_Gate) |
| MC.07.04.x02 | bug | widgets | The reporter said widgets could not be viewed while menu icons were hidden because clicking the Notification Center button did nothing. | T2 [GitHub jacklandrin/OnlySwitch](https://github.com/jacklandrin/OnlySwitch/issues/218) |
| MC.07.04.x03 | new | widgets | Desktop widgets can be made much larger using a new Extra-Large Portrait option in their context menu. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MC.07.04.x04 | behavior | widgets | Widgets update in real time when their associated app is open. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MC.07.04.x05 | behavior | widgets | Widgets in Notification Center are not directly interactive; selecting one can deep-link to a related feature in its app. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |
| MC.07.04.x06 | behavior | widgets | Open Notification Center and select “Edit Widgets” to open the widget gallery. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |
| MC.07.04.x07 | behavior | widgets | The widget gallery’s “All Apps” view lists available widgets from supported apps installed on the Mac and Apple devices; the “Search Widgets” box can be used to find an app. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |
| MC.07.04.x08 | behavior | widgets | Add a widget by selecting it or its green plus icon, or by dragging it into Notification Center. Widgets can also be added to the desktop; select “Done” to close the widget window. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |
| MC.07.04.x09 | behavior | widgets | To import widgets from an iPhone or iPad app or data, choose the app in the gallery sidebar, open the “From iPhone” or “iPad” tab, select a widget, and choose “Done”. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |
| MC.07.04.x10 | behavior | widgets | Rearrange Notification Center widgets by dragging them up or down. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |
| MC.07.04.x11 | behavior | widgets | Right-click a widget and select “Edit [Widget Name]” to customize its information; select “Done” after making changes. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |
| MC.07.04.x12 | behavior | widgets | Right-click a widget and choose “Small”, “Medium”, or “Large” to change its size. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |
| MC.07.04.x13 | behavior | widgets | Right-click a widget and select “Remove Widget” to remove it. The widget gallery preview also has delete icons for removing multiple widgets. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |

## 14.8 MC.08 Dialogs & panels


### MC.08.01 Open/Save panels (Recents Cmd-Shift-F, 27)

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).

Also covered by: [macOS 27 Golden Gate Release Notes](https://developer.apple.com/documentation/macos-release-notes/macos-27-release-notes)


#### Open documents on Mac - Apple Support

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchl971293e1/27/mac/27) · macOS 27 text present: no

The page describes ways to open documents on a Mac using the desktop, search, an associated app, or Finder. It also notes that iCloud Drive must be set up before opening documents stored there and that Pages can open Microsoft Office files.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.08.01.e001 | desktop | view | Mac desktop | A place where a document’s icon may appear. |
| MC.08.01.e002 | document icon | icon | Desktop or Finder | Double-click to open the document. |
| MC.08.01.e003 | menu bar | other | Mac menu bar | The page says to click the menu bar before entering a document name in the search field. |
| MC.08.01.e004 | search field | control | After clicking the menu bar | Enter a document’s name to find it. |
| MC.08.01.e005 | results | view | Search results | Double-click a document in the results to open it. |
| MC.08.01.e006 | Apple menu | menu | Mac menu bar | The page says to choose it if you recently worked in a document; no further menu choice is included in the provided text. |
| MC.08.01.e007 | associated app | other | App used for the document | Open the app associated with the document. |
| MC.08.01.e008 | Open dialog | window | Associated app | Choose the document in this dialog, if available. |
| MC.08.01.e009 | File > Open | menu item | Associated app | Menu command for opening a document. |
| MC.08.01.e010 | File > Open Recent | menu item | Some apps | Menu command for opening recently used documents. |
| MC.08.01.e011 | Finder sidebar | view | Finder | Click a listed location or folder to find a document. |
| MC.08.01.e012 | Recents | other | Finder sidebar | Click to browse recent documents. |
| MC.08.01.e013 | iCloud Drive | other | Finder sidebar | Click to browse documents in iCloud Drive. |
| MC.08.01.e014 | Documents | other | Finder sidebar | Click to browse the Documents location. |
| MC.08.01.e015 | folder | other | Finder sidebar or document location | Click the folder where the document is located. |
| MC.08.01.e016 | document’s icon or name | other | Finder | Double-click to open the document. |
| MC.08.01.e017 | Pages app | other | Mac | The page recommends using Pages to open files created with Microsoft Office. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.08.01.b001 | Double-click a document’s icon on the desktop. | The document opens. | The document’s icon is on the desktop. |
| MC.08.01.b002 | Click the menu bar, enter the document’s name in the search field, then double-click the document in the results. | The document opens. | — |
| MC.08.01.b003 | Choose Apple menu. | Not stated in the provided text. | You recently worked in a document. |
| MC.08.01.b004 | Open the document’s associated app, then choose the document in the Open dialog or choose File > Open. | The document opens. | The Open dialog is available for the app, or the app provides File > Open. |
| MC.08.01.b005 | Choose File > Open Recent, then choose a document. | A recently used document opens. | Available in some apps. |
| MC.08.01.b006 | Click Recents, iCloud Drive, Documents, or the folder where the document is located in the Finder sidebar, then double-click the document’s icon or name. | The document opens. | To open documents in iCloud Drive, iCloud Drive must first be set up. |
| MC.08.01.b007 | Use the Pages app to open a file created with Microsoft Office. | The file can be opened on the Mac. | — |

#### Create and work with documents on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchldc1dd114/27/mac/27) · macOS 27 text present: no

The page explains how to create, format, save, and work with documents using Mac apps such as Pages, Numbers, Keynote, and TextEdit.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.08.01.e018 | TextEdit | other | App example in Create documents | An app that can create plain text, rich text, or HTML documents. |
| MC.08.01.e019 | Open dialog | window | When opening an app that creates documents | The dialog where you can click New Document. |
| MC.08.01.e020 | New Document | button | Open dialog | Starts a new document. |
| MC.08.01.e021 | File | menu | Menu bar in a document app | Contains commands including New, Save, Save As, and Duplicate. |
| MC.08.01.e022 | New | menu item | File menu | Creates a new document. |
| MC.08.01.e023 | Pages | other | Create documents | An app for creating letters, reports, flyers, posters, and more; includes templates. |
| MC.08.01.e024 | Numbers | other | Create documents | An app for creating spreadsheets. |
| MC.08.01.e025 | Keynote | other | Create documents | An app for creating presentations. |
| MC.08.01.e026 | App Store | other | Create documents | Where Pages, Numbers, and Keynote can be obtained if they are not on the Mac. |
| MC.08.01.e027 | Open App Store for me | other | Link in Create documents | Link to open the App Store. |
| MC.08.01.e028 | Help | menu | Menu bar while working in an app | Provides access to the app’s user guide and answers. |
| MC.08.01.e029 | Format | menu | Menu bar in a document app | Contains text formatting commands. |
| MC.08.01.e030 | Show Fonts | menu item | Format menu, or Format > Font submenu | Opens font controls. |
| MC.08.01.e031 | Font | menu | Format menu | Contains the Show Fonts command. |
| MC.08.01.e032 | Style | menu item | Format menu | Provides style controls. |
| MC.08.01.e033 | Show Colors | menu item | Format menu, or Format > Font submenu | Opens color controls. |
| MC.08.01.e034 | Look Up | menu item | Context menu after Control-clicking selected text | Looks up the selected text. |
| MC.08.01.e035 | Translate | menu item | Context menu after Control-clicking selected text | Translates the selected text. |
| MC.08.01.e036 | Save | menu item | File menu | Saves the document. |
| MC.08.01.e037 | Save As | menu item | File menu | Saves the document under another name, or as a copy. |
| MC.08.01.e038 | Option | other | Keyboard; used while opening the File menu | Holding this key makes Save As available when it is not shown. |
| MC.08.01.e039 | Duplicate | menu item | File menu | Saves a document as a copy. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.08.01.b008 | Open an app that lets you create documents. | You can begin creating a document in that app. | The app must support document creation. |
| MC.08.01.b009 | Open TextEdit. | You can create a plain text, rich text, or HTML document. | TextEdit is the app being used. |
| MC.08.01.b010 | Click New Document in the Open dialog, or choose File > New. | A new document is created. | In an app that supports document creation. |
| MC.08.01.b011 | Use Pages. | You can create letters, reports, flyers, posters, and more; templates are available. | Pages is available on the Mac or obtained from the App Store. |
| MC.08.01.b012 | Start with a template in Numbers. | You can modify the spreadsheet by adding formulas, charts, images, and more. | Numbers is being used. |
| MC.08.01.b013 | Use Keynote. | You can create presentations with images, media, charts, and slide animations. | Keynote is available on the Mac or obtained from the App Store. |
| MC.08.01.b014 | Get Pages, Numbers, or Keynote from the App Store. | The app can be installed if it is not already on the Mac. | The app is not on the Mac. |
| MC.08.01.b015 | Use Pages, Numbers, or Keynote on an iOS or iPadOS device, or on iCloud.com. | Those apps are also available through those platforms. | For iOS and iPadOS, get them from the App Store. |
| MC.08.01.b016 | Choose Help in the menu bar while working in an app. | Look for answers in the app’s user guide. | While working in the app. |
| MC.08.01.b017 | Choose Format > Show Fonts, Format > Font > Show Fonts, or Format > Style. | Change fonts and styles in the document. | In a document. |
| MC.08.01.b018 | Choose Format > Show Colors or Format > Font > Show Colors. | Change colors in the document. | In a document. |
| MC.08.01.b019 | Enter characters with accent marks or diacritic marks. | Characters with those marks are entered in the document. | In a document. |
| MC.08.01.b020 | Type in an app that checks spelling. | Spelling is checked while you type, and mistakes are automatically corrected. | In most apps; the features can be turned off or other options used. |
| MC.08.01.b021 | Select text, Control-click it, then choose Look Up. | The selected text is looked up. | In a document. |
| MC.08.01.b022 | Select text, Control-click it, then choose Translate. | The selected text is translated. | In a document. |
| MC.08.01.b023 | Work in an app that saves documents automatically. | The document is saved automatically while you work. | Many, but not all, apps on Mac save automatically. |
| MC.08.01.b024 | Choose File > Save, enter a name, and choose where to save the document. | The document is saved. | The page’s instructions about showing more save locations are incomplete in the supplied text. |
| MC.08.01.b025 | Save a document. | You can add tags to make the document easier to find later. | When saving the document. |
| MC.08.01.b026 | Save a document in iCloud Drive. | It can be available on your computers and iOS and iPadOS devices set up with iCloud Drive. | You may be able to choose iCloud Drive as the save location; the devices must be set up with iCloud Drive. |
| MC.08.01.b027 | Choose File > Save As and enter a new name. | The document is saved with another name. | If Save As is not shown, press and hold Option, then open the File menu again. |
| MC.08.01.b028 | Choose File > Duplicate or File > Save As. | The document is saved as a copy. | In an app with the relevant File menu command. |
| MC.08.01.b029 | Save a document as a PDF. | A PDF version of the document is saved. | The page does not specify further steps. |
| MC.08.01.b030 | Combine multiple files into a single PDF. | The files are combined into one PDF. | The page does not specify further steps. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.08.01.k001 | Option | Press and hold Option, then open the File menu again to reveal Save As when it is not shown. | In a document app. |

#### File management

T1 · HIG (developer guidance) · [Apple HIG](https://developer.apple.com/design/human-interface-guidelines/file-management) · macOS 27 text present: no

Guidance for macOS document creation, opening, saving, file browsing, Quick Look, Finder Sync, and communicating unsaved changes.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.08.01.e040 | File | menu | macOS menu bar | Contains familiar commands for creating and opening documents; the add action belongs here. |
| MC.08.01.e041 | New | menu item | File menu | Command for creating a new document. |
| MC.08.01.e042 | Open | menu item | File menu | Command for opening an existing document. |
| MC.08.01.e043 | Add (+) | button | A macOS app’s File menu | Action that helps people create a new document. |
| MC.08.01.e044 | Finder | other | macOS | The familiar file-browsing experience people use to access the macOS file system. |
| MC.08.01.e045 | Open | button | macOS open panel | Its title can be customized to reflect the task, such as changing it to Insert. |
| MC.08.01.e046 | Untitled | other | New document title | Default title until people choose a custom name. |
| MC.08.01.e047 | Save dialog | panel | macOS save interface | Can include a custom accessory view with useful settings or options. |
| MC.08.01.e048 | Ask to keep changes when closing documents | setting | Desktop & Dock settings | Toggle that turns off autosaving. |
| MC.08.01.e049 | close button | button | Document window | When autosaving is off, show a dot on this button to indicate unsaved changes. |
| MC.08.01.e050 | Window | menu | App menu bar | When autosaving is off, show a dot next to the document’s name in this menu if it has unsaved changes. |
| MC.08.01.e051 | Edited | other | Document title in the title bar | Optional suffix indicating edits; remove it after autosaving or an explicit save. |
| MC.08.01.e052 | Quick Look viewer | view | Within an app | Lets people preview files, including files the app can’t open. |
| MC.08.01.e053 | Quick Look generator | other | Used by Finder, Files, and Spotlight | Can provide previews of an app’s custom file types. |
| MC.08.01.e054 | Finder Sync extension | other | Finder | Can display sync-status badges and provide custom contextual menu items and toolbar buttons. |

| ID | Context | Guideline | Conditions |
|---|---|---|---|
| MC.08.01.b031 | People create or open documents in a macOS app. | Provide familiar menu commands such as New and Open in the File menu. | For macOS apps. |
| MC.08.01.b032 | People need to create a document. | Provide an Add (+) button; in a macOS app, put the add action in the File menu. | Regardless of keyboard-shortcut availability. |
| MC.08.01.b033 | An app provides a custom file browser. | Use the default file browser unless there is an important reason to create a custom one; let people browse the rest of the file system even if the browser opens to a particularly relevant location. | The custom browser may initially show a location such as Documents, iCloud, or the most recently selected location. |
| MC.08.01.b034 | People edit a document or close it or switch to another app. | Automatically save periodically during editing and when the file is closed or the person switches apps, rather than requiring an explicit save action. | General saving guidance. |
| MC.08.01.b035 | An app displays save or open interfaces. | Hide file extensions by default while allowing people to view them, and reflect their current choice in every save or open interface. | — |
| MC.08.01.b036 | An app lets people preview files. | Use a Quick Look viewer to preview files the app can’t open; consider a Quick Look generator for custom file types so other apps can display previews. | Generators can provide previews in Finder, Files, and Spotlight. |
| MC.08.01.b037 | An app provides a custom macOS file-opening interface. | Make it convenient with options such as Open Recent, filtering, or selecting multiple documents; customize the Open button title when that better describes the task. | For example, use Insert when inserting a file’s contents into the current document. |
| MC.08.01.b038 | People save a document. | Provide an interface for changing the file’s name, format, or location, and provide a way to choose a format when the app supports multiple formats. | A new document’s default title is Untitled; the save view can default to a logical location. |
| MC.08.01.b039 | A save dialog could benefit from app-specific options. | Consider adding a custom accessory view with useful settings or options. | For example, Mail’s save dialog offers an option to include attachments. |
| MC.08.01.b040 | An app syncs local and remote files. | Consider a Finder Sync extension to show synchronization status in Finder and provide file-management contextual menu items or global-action toolbar buttons. | Examples include status badges, favoriting, password-protection, and initiating a sync. |
| MC.08.01.b041 | People turn off autosaving using the “Ask to keep changes when closing documents” toggle. | Show that documents have unsaved changes and present a save dialog when people close a document, quit the app, log out, or restart. | Autosaving is off. |
| MC.08.01.b042 | A document has unsaved changes while autosaving is off. | Show a dot on the document window’s close button and next to the document’s name in the app’s Window menu. | Do not show these dots when autosaving is on, because they imply action is needed to avoid losing work. |
| MC.08.01.b043 | A document title indicates edits. | The app may append “Edited” to the title bar; remove the suffix as soon as autosaving occurs or the person explicitly saves. | Regardless of autosave status. |

System Settings paths referenced: Desktop & Dock

#### Independent reports (MC.08.01)

| ID | Kind | Area | Claim | Source |
|---|---|---|---|---|
| MC.08.01.x01 | new | open/save | The document save option in the menu bar can scan a file’s contents and offer filename suggestions. | T4 [Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/) |
| MC.08.01.x02 | behavior | open/save | Return activates the main full-colour button in a form, regardless of which element currently has focus; the article gives Save in a document save dialog as an example. | T4 [tempertemper](https://www.tempertemper.net/blog/using-the-keyboard-to-navigate-on-macos) |
| MC.08.01.x03 | new | open/save | The Recents list in open and save panels can be accessed with the keyboard shortcut Command-Shift-F. | T5 [MacRumors Forums](https://forums.macrumors.com/threads/macos-27-all-the-little-things.2483520/) |
| MC.08.01.x04 | behavior | open/save | The Window > Move & Resize and Window > Full Screen Tile menus can move or resize open and save panels that are not displayed as sheets. | T5 [MacRumors Forums](https://forums.macrumors.com/threads/macos-27-all-the-little-things.2483520/) |
| MC.08.01.x05 | new | open/save | The Recents list in open and save panels can be accessed with the keyboard shortcut cmd-shift-f. | T4 [BetaProfile](https://betaprofile.dev/macos-27-rc/) |

### MC.08.02 Print

Status: **PARTIAL**. Mac User Guide print page captured; HIG Printing page inaccessible (GAP-B5-11).


#### Print documents on your Mac - Apple Support

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mh35838/27/mac/27) · macOS 27 text present: no

Explains how to set up a printer, print a document, check a print job, and save a document as a PDF on a Mac.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.08.02.e001 | Before you begin | other | Section heading | Introduces printer setup requirements. |
| MC.08.02.e002 | Print a document | other | Page card and section heading | Topic for sending a document to a printer. |
| MC.08.02.e003 | Save as PDF | other | Page card and section heading | Topic for saving a document as a PDF. |
| MC.08.02.e004 | System Settings | other | Before you begin | Mentioned as a place where printer software may be available when adding a printer. |
| MC.08.02.e005 | Add a printer | other | Before you begin | Apple Support article referenced for more information. |
| MC.08.02.e006 | Identify the ports on your Mac | other | Before you begin | Apple Support article referenced for information about Mac ports. |
| MC.08.02.e007 | About AirPrint | other | Before you begin | Apple Support article referenced for checking AirPrint support. |
| MC.08.02.e008 | Choose print options | other | Print a document | Apple Support article referenced for information about print choices. |
| MC.08.02.e009 | Print | button | Print dialog | Clicking this sends the document to the printer. |
| MC.08.02.e010 | Dock | other | While printing | The printer icon appears here while a print job is in progress. |
| MC.08.02.e011 | Preview a file | other | Want to learn more? | Related link about reading, annotating, and signing PDFs in Preview. |
| MC.08.02.e012 | Keyboard shortcuts | other | Want to learn more? | Related link about shortcuts for everyday tasks. |
| MC.08.02.e013 | Organize your files | other | Want to learn more? | Related link about organizing documents in Finder. |
| MC.08.02.e014 | Learn more | other | Want to learn more? links | Link label shown with the related topics. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.08.02.b001 | Connect a printer to the Mac with a cable. | The printer can be set up to work with the Mac. | For a printer that connects to the Mac; a suitable cable and possibly an adapter may be needed. |
| MC.08.02.b002 | Install the printer driver. | The software that allows the printer to work with the Mac is installed. | The driver is typically available from the printer manufacturer’s website and may sometimes be available when adding the printer in System Settings. |
| MC.08.02.b003 | Set up an AirPrint printer on the same network as the Mac. | The wireless printer is set up to work with the Mac. | The printer must support AirPrint. |
| MC.08.02.b004 | Open a document in an app and use the print command. | The document can be sent to the printer. | The printer must first be set up to work with the Mac; printing is described as available from almost any app. |
| MC.08.02.b005 | Choose print options. | Adjust paper size and orientation, select color or black and white, and choose whether to print double-sided. | Color printing depends on printer support. |
| MC.08.02.b006 | Click Print. | The print job is sent to the printer. | A document is open in an app and a printer is set up. |
| MC.08.02.b007 | Click the printer icon that appears in the Dock. | Check the status of the print job. | The icon appears while printing. |
| MC.08.02.b008 | Save a document as a PDF using an app’s print feature. | Save the document as a PDF; the resulting PDF looks as it would when printed and can be shared, opened in Preview, or viewed on other devices. | The app must support printing; no printer is needed. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.08.02.k001 | Command (shown without a following key) | Print a document. | The page says to open a document in an app and press Command; the shortcut is incomplete in the supplied text. |
| MC.08.02.k002 | Command (shown without a following key) | Save a document as a PDF. | The page says to press Command; the shortcut is incomplete in the supplied text. |

### MC.08.03 Share

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).


#### Use the Share menu on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mh40614/27/mac/27) · macOS 27 text present: no

The Share menu in Finder and many Mac apps offers ways to share selected items. Its available options depend on the item and the sharing method. The page also describes how to change which sharing extensions are included.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.08.03.e001 | Share menu | menu | Finder and many apps on Mac | Offers ways to share selected items. |
| MC.08.03.e002 | Share icon | icon | In Finder or an app, near the selected item | The page says to click this icon; no text label for the icon is provided. |
| MC.08.03.e003 | Share | menu item | Shortcut menu for an item on the desktop | Opens sharing options. |
| MC.08.03.e004 | AirDrop | menu item | Share menu | An example of a way to share an item. |
| MC.08.03.e005 | Mail | menu item | Share menu | An example of an app used to share an item. |
| MC.08.03.e006 | Messages | menu item | Share menu | An example of an app used to share an item. |
| MC.08.03.e007 | Set Wallpaper | menu item | Share menu when an image is selected in Photos | Sets the image as the desktop picture. |
| MC.08.03.e008 | Options | button | Share menu when sharing an .heic file | Provides options for the file format. |
| MC.08.03.e009 | Format | control | Options for sharing an .heic file | A pop-up menu for choosing the format behavior. |
| MC.08.03.e010 | Automatic | menu item | Format pop-up menu | Allows the best format for the destination. |
| MC.08.03.e011 | Current | menu item | Format pop-up menu | Prevents the file from being converted to another format. |
| MC.08.03.e012 | Most Compatible | menu item | Format pop-up menu | Allows photos and videos to be converted to .jpeg, .png, and H.264 formats. |
| MC.08.03.e013 | Collaborate | menu item | Share menu for a document created in an app that supports collaboration | Sends an invitation to work together on the document. |
| MC.08.03.e014 | Send Copy | menu item | Share menu for a document created in an app that supports collaboration | Sends the document. |
| MC.08.03.e015 | Edit Extensions | menu item | Share menu | Used to customize the Share menu. |
| MC.08.03.e016 | System Settings | app | Mac | Contains the settings used to change sharing extensions. |
| MC.08.03.e017 | General | setting | System Settings | Settings category containing Login Items & Extensions. |
| MC.08.03.e018 | Login Items & Extensions | setting | System Settings > General | Settings area containing Extensions. |
| MC.08.03.e019 | Open Login Items & Extensions settings for me | other | Near the Login Items & Extensions instructions | Link text provided on the page. |
| MC.08.03.e020 | Extensions | setting | System Settings > General > Login Items & Extensions | Contains sharing extensions. |
| MC.08.03.e021 | By Category | control | Extensions settings | A category option to click. |
| MC.08.03.e022 | sharing extensions | setting | Extensions settings | Can be turned on or off to control inclusion in the Share menu. |
| MC.08.03.e023 | Done | button | Extensions settings | Clicked after choosing which sharing extensions to include. |
| MC.08.03.e024 | Photos | app | Mac | The page gives selecting an image in Photos as an example. |
| MC.08.03.e025 | Safari | app | Mac | The page gives sharing a webpage in Safari as an example. |
| MC.08.03.e026 | Finder | app | Mac | The Share menu is available in Finder. |
| MC.08.03.e027 | Control-click | gesture | An item on the desktop | Opens the shortcut menu for the item. |
| MC.08.03.e028 | shortcut menu | menu | An item on the desktop after Control-click | Contains the Share command. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.08.03.b001 | Select one or more items in Finder or an app, then click the Share icon. | The Share menu opens. | The item or items are selected. |
| MC.08.03.b002 | Control-click an item on the desktop, then choose Share from the shortcut menu. | The Share menu opens for the item. | The item is on the desktop. |
| MC.08.03.b003 | Choose a sharing option in the Share menu. | The selected item is shared using that method. | Available options depend on the type of item. |
| MC.08.03.b004 | Select an image in Photos and choose Set Wallpaper. | The image is set as the desktop picture. | The selected item is an image in Photos. |
| MC.08.03.b005 | When sharing an .heic file, click Options, click the Format pop-up menu, and choose Automatic. | The best format for the destination is allowed. | The item being shared is an .heic file. |
| MC.08.03.b006 | When sharing an .heic file, click Options, click the Format pop-up menu, and choose Current. | The file is prevented from being converted to another format. | The item being shared is an .heic file. |
| MC.08.03.b007 | When sharing an .heic file, click Options, click the Format pop-up menu, and choose Most Compatible. | Photos and videos are allowed to be converted to .jpeg, .png, and H.264 formats. | The item being shared is an .heic file. |
| MC.08.03.b008 | Choose Collaborate. | An invitation to work together on the document is sent. | The document was created in an app that supports collaboration. |
| MC.08.03.b009 | Choose Send Copy. | The document is sent as a copy. | The document was created in an app that supports collaboration. |
| MC.08.03.b010 | Choose Edit Extensions. | The Share menu can be customized. | The option is chosen from the Share menu. |
| MC.08.03.b011 | A sharing method asks for additional information. | Provide the requested information. | Only if asked. |
| MC.08.03.b012 | Go to System Settings, click General, click Login Items & Extensions, go to Extensions, click By Category, and turn on sharing extensions. | The selected sharing extensions are included in the Share menu when applicable. | The user turns on the desired sharing extensions and clicks Done. |
| MC.08.03.b013 | Turn off a sharing extension in Extensions settings. | That extension is not included in the Share menu. | The user clicks Done after changing the selection. |
| MC.08.03.b014 | Try to share an item using an extension that is turned on. | The extension may not appear in the Share menu. | The item cannot be shared using that extension. |
| MC.08.03.b015 | Share a webpage in Safari using AirDrop. | The webpage can be shared using AirDrop. | The selected item is a webpage in Safari. |
| MC.08.03.b016 | Try to share selected text in Safari using AirDrop. | The selected text cannot be shared using AirDrop. | The item being shared is selected text. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.08.03.k001 | Control-click | Open the shortcut menu for an item. | An item on the desktop. |

System Settings paths referenced: System Settings > General > Login Items & Extensions > Extensions > By Category

#### Activity views

T1 · HIG (developer guidance) · [Apple HIG](https://developer.apple.com/design/human-interface-guidelines/activity-views) · macOS 27 text present: no

The page says macOS does not provide an activity view, but people can use share and action extensions on Mac. It describes where those extensions are accessed and gives guidance for designing and handling them.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.08.03.e029 | Share button | button | macOS toolbar | People can click this button to access share extensions. |
| MC.08.03.e030 | Share | menu item | macOS context menu | Choosing this item provides access to share extensions. |
| MC.08.03.e031 | Action extension | other | Certain embedded content, a toolbar button, or a Finder window | A content-specific extension that people can access by hovering over certain embedded content, clicking a toolbar button, or choosing a quick action in Finder. |
| MC.08.03.e032 | Quick action | menu item | Finder window | A way to access an action extension. |
| MC.08.03.e033 | Composition view | view | Share extension | The system-provided view that the page recommends using for a familiar sharing experience. |
| MC.08.03.e034 | Modal view | view | Extension interface | The system displays an extension within a modal view by default; the page advises against placing another modal view above it. |

| ID | Context | Guideline | Conditions |
|---|---|---|---|
| MC.08.03.b017 | A person wants to use a share extension on macOS. | They can click a Share button in the toolbar or choose Share in a context menu. | macOS does not provide an activity view. |
| MC.08.03.b018 | A person wants to use an action extension on macOS. | They can hold the pointer over certain embedded content, click a toolbar button, or choose a quick action in a Finder window. | The available access method depends on the context and content. |
| MC.08.03.b019 | Designing a share extension. | Prefer the system-provided composition view for a consistent sharing experience. | — |
| MC.08.03.b020 | Designing an action extension. | Include the app name; if an interface is needed, include elements of the app’s interface to help people recognize the relationship. | — |
| MC.08.03.b021 | Designing an extension interaction. | Streamline and limit interaction so people can complete the task in a few steps. | — |
| MC.08.03.b022 | An extension needs to present an alert or another interface. | Avoid placing a modal view above the extension; an alert may be necessary. | The system displays an extension within a modal view by default. |
| MC.08.03.b023 | An action extension needs an image to communicate its purpose. | Prefer a symbol or create an interface icon that clearly identifies the task. | Share extensions automatically use the app icon. |
| MC.08.03.b024 | A share or action extension performs a lengthy operation. | Continue the task in the background and provide a way to check its status in the main app. | The extension dismisses immediately after the task is completed. A notification may report a problem, but should not be sent solely to announce completion. |

API names: `UIActivity`, `App Extension Support`

### MC.08.04 Services menu

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).


#### Use services in apps on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp1012/27/mac/27) · macOS 27 text present: no

Services let you use features of other apps for the task you’re doing without opening those apps. The page explains how to access and configure services, and what to try when a service can’t be used.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.08.04.e001 | Apple menu | menu | Menu bar | The app menu is described as being to the right of the Apple menu. |
| MC.08.04.e002 | app menu | menu | Menu bar, to the right of the Apple menu | The page says to open this menu when using a service; the remaining instruction is not present in the supplied text. |
| MC.08.04.e003 | Services menu | menu | An app’s menu | Contains services available for the app or task. |
| MC.08.04.e004 | System Settings | app | Mac | Open this app to change the services available in apps. |
| MC.08.04.e005 | Keyboard | setting | System Settings | Click this setting to access the Services option. |
| MC.08.04.e006 | Services | setting | System Settings > Keyboard sidebar | Select or deselect items to make them available in, or remove them from, the Services menu. |
| MC.08.04.e007 | Open Keyboard settings for me | other | Under the instruction to click Keyboard | Link text provided on the page. |
| MC.08.04.e008 | File | menu | An app’s menu bar | Choose this menu when restoring an app from the Trash. |
| MC.08.04.e009 | Put Back | menu item | File menu | Choose this item to restore an app from the Trash. |
| MC.08.04.e010 | sidebar | view | System Settings > Keyboard | The page says to click Services in the sidebar. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.08.04.b001 | Select an item, such as a file on the desktop or text in a document. | The selected item is the subject of the service task. | On Mac; the page’s next instruction to open the app menu is incomplete in the supplied text. |
| MC.08.04.b002 | Open the app menu to the right of the Apple menu. | The page’s instruction is incomplete; it does not state what to choose next in the supplied text. | While using a service. |
| MC.08.04.b003 | Go to System Settings and click Keyboard, then click Services in the sidebar. | The Services settings are available for changing which services appear in apps. | On Mac. |
| MC.08.04.b004 | Select an item in the Services settings. | The item is made available in the Services menu. | The item appears in an app’s Services menu only when relevant to that app or task. |
| MC.08.04.b005 | Deselect an item in the Services settings. | The item is removed from the Services menu. | In System Settings > Keyboard > Services. |
| MC.08.04.b006 | Create a keyboard shortcut for a service. | You can use the service without using an app’s Services menu. | The page does not specify a particular shortcut or creation procedure. |
| MC.08.04.b007 | Try to use a service when you don’t have permission to open the app providing it. | Log in as an administrator to use the service. | The app providing the service requires permission to open. |
| MC.08.04.b008 | A service’s app is missing or can’t be opened. | Search for the app on your Mac and open it, then try the service again. | The app is missing or can’t be opened. |
| MC.08.04.b009 | The app providing a service is in the Trash. | Select the app and choose File > Put Back, then try the service again. | The app is in the Trash. |

System Settings paths referenced: System Settings > Keyboard > Services

### MC.08.05 Alerts

Status: **PARTIAL**. Only HIG guidance (developer, not OS behaviour) and the NSAlert release-note item.


#### Alerts

T1 · HIG (developer guidance) · [Apple HIG](https://developer.apple.com/design/human-interface-guidelines/alerts) · macOS 27 text present: no

Apple’s guidance for alerts: use them sparingly for important, actionable information; keep their content and buttons clear; and provide safe ways to cancel consequential actions.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.08.05.e001 | alert | view | Modal view | Displays critical information, such as a problem or a warning about an action that might destroy data. |
| MC.08.05.e002 | title | other | Alert | Clearly and succinctly describes the situation. |
| MC.08.05.e003 | informative text | other | Alert | Optional text that adds value; keep it brief. |
| MC.08.05.e004 | buttons | button | Alert | An alert can have up to three buttons. Use succinct titles that describe the result of selecting them. |
| MC.08.05.e005 | text field | control | Alert | Optional; include it only when input is needed to resolve the situation. |
| MC.08.05.e006 | app icon | icon | macOS alert | macOS displays the app icon automatically; an alternative icon or symbol can be supplied. |
| MC.08.05.e007 | icon | icon | macOS alert | An alternative icon or symbol can be supplied. Use a caution symbol sparingly. |
| MC.08.05.e008 | accessory view | view | macOS alert | A custom view that can be appended when additional information is necessary. |
| MC.08.05.e009 | suppression checkbox | control | macOS alert | Can let people suppress subsequent occurrences of a repeating alert. |
| MC.08.05.e010 | Help button | button | macOS alert | Opens help documentation. |
| MC.08.05.e011 | OK | button | Informational alert | An acceptable button title for acceptance in informational alerts; avoid it as the default for other alerts. |
| MC.08.05.e012 | Cancel | button | Alert | The required title for a button that cancels the alert’s action. It should not be the default button. |
| MC.08.05.e013 | Done | button | Single-button alert | Use when the single button is also the default button. |
| MC.08.05.e014 | View All | button | Alert button-title example | Example of a succinct button title. |
| MC.08.05.e015 | Reply | button | Alert button-title example | Example of a succinct button title. |
| MC.08.05.e016 | Ignore | button | Alert button-title example | Example of a succinct button title. |
| MC.08.05.e017 | Erase | button | Alert button-title example | Example of a specific title for a consequential action. |
| MC.08.05.e018 | Convert | button | Alert button-title example | Example of a specific title for a consequential action. |
| MC.08.05.e019 | Clear | button | Alert button-title example | Example of a specific title for a consequential action. |
| MC.08.05.e020 | Delete | button | Alert button-title example | Example of a specific title for a consequential action. |
| MC.08.05.e021 | exclamationmark.triangle | icon | macOS alert | Example caution symbol; use only when extra attention is needed. |
| MC.08.05.e022 | indicator | indicator | Mail example | An alternative to an informational alert when a server connection is unavailable; people can choose it to learn more. |

| ID | Context | Guideline | Conditions |
|---|---|---|---|
| MC.08.05.b001 | When an alert would interrupt the current task | Use alerts sparingly, and ensure each offers only essential information and useful actions. | General alert guidance. |
| MC.08.05.b002 | When information is not actionable | Avoid using an alert merely to provide information; communicate it within the relevant context when possible. | General alert guidance. |
| MC.08.05.b003 | When an action is common, destructive, and undoable | Avoid displaying an alert. | The guidance gives deleting an email or file as examples. |
| MC.08.05.b004 | When an action is uncommon, destructive, and cannot be undone | Display an alert so people can catch an accidental action. | General alert guidance. |
| MC.08.05.b005 | When the app starts | Avoid showing an alert; make important information discoverable or use a nonintrusive alternative. | For a startup problem such as no network connection, the page suggests showing cached or placeholder data with a nonintrusive label. |
| MC.08.05.b006 | When writing alert copy | Use direct, neutral, approachable language; make the title clear and succinct, and include informative text only when it adds value. | Keep titles from becoming overly long; informative text should be brief. |
| MC.08.05.b007 | When naming alert buttons | Use succinct, logical titles describing the result; use title-style capitalization and no ending punctuation. | Prefer verbs or verb phrases. Use OK only for acceptance in informational alerts; avoid Yes and No. |
| MC.08.05.b008 | When an alert asks someone to confirm an action | Avoid using OK as the default button title; use a specific action title. | OK can be unclear about what the person is agreeing to. |
| MC.08.05.b009 | When placing alert buttons | Place the most likely choice and the default button on the trailing side in a row, or at the top in a stack. Place Cancel on the leading side in a row, or at the bottom in a stack. | General button-placement guidance. |
| MC.08.05.b010 | When a button performs a destructive action the person did not deliberately choose | Use the destructive style to identify that button. | Do not apply the destructive style when the alert follows a destructive action the person deliberately chose, such as Empty Trash. |
| MC.08.05.b011 | When an alert includes a destructive action | Include a Cancel button and do not make it the default. | If encouraging people to read the alert, avoid making any button the default. If a single button must also be the default, use Done, not Cancel. |
| MC.08.05.b012 | When it makes sense to offer another way to cancel an alert | Support Escape or Command-Period on an attached keyboard. | Listed for iOS, iPadOS, macOS, and visionOS. |
| MC.08.05.b013 | When designing an alert for macOS | The system displays the app icon automatically; an alternative icon or symbol can be supplied. macOS alerts can also have repeating-alert suppression, a custom accessory view, and a Help button. | A custom view is for cases where additional information is necessary. |
| MC.08.05.b014 | When considering a caution symbol in a macOS alert | Use it sparingly, only when extra attention is needed. | The page says not to use it for tasks whose only purpose is overwriting or removing data, such as saving or emptying Trash. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.08.05.k001 | Escape (Esc) | Cancel or dismiss an onscreen alert. | On an attached keyboard; listed for iOS, iPadOS, macOS, and visionOS. |
| MC.08.05.k002 | Command-Period (.) | Cancel or dismiss an onscreen alert. | On an attached keyboard; listed for iOS, iPadOS, macOS, and visionOS. |

API names: `alert(_:isPresented:`, `UIAlertController`, `accessory`

#### Independent reports (MC.08.05)

| ID | Kind | Area | Claim | Source |
|---|---|---|---|---|
| MC.08.05.x01 | behavior | alerts | In a dialog with Cancel and Save, Cancel has a highlighted outline and focus. Pressing Space activates Cancel, while Return activates the submit button, Save. | T4 [tempertemper](https://www.tempertemper.net/blog/using-the-keyboard-to-navigate-on-macos) |
| MC.08.05.x02 | bug | alerts | Fixed an issue where NSAlert buttons incorrectly allowed bezelColor customization. | T4 [BetaProfile](https://betaprofile.dev/macos-27-rc/) |

## 14.9 MC.09 Input


### MC.09.01 Keyboard shortcuts

Status: **COMPLETE**. 208 system shortcuts parsed from Apple KB 102650 (published 2026-09-14) and the tiling-shortcut page; app-specific shortcuts are out of scope (MC.03/MD.30).

Also covered by: [Mac keyboard shortcuts - Apple Support](https://support.apple.com/en-us/102650)


#### Use macOS keyboard shortcuts

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp2262/27/mac/27) · macOS 27 text present: no

Explains how to use, customize, and disable macOS keyboard shortcuts, and notes that shortcuts can vary by app, language, and keyboard layout.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.09.01.e001 | Apple menu | menu | At the start of the Keyboard settings instructions | Menu to choose when opening Keyboard settings. |
| MC.09.01.e002 | Open Keyboard settings for me | other | After the Apple menu references in the instructions | Link labeled “Open Keyboard settings for me.” |
| MC.09.01.e003 | Keyboard settings | window | Opened from the Apple menu | Settings area used to customize or disable keyboard shortcuts. |
| MC.09.01.e004 | Mission Control | other | Keyboard settings sidebar | Example category for selecting keyboard shortcuts. |
| MC.09.01.e005 | Spotlight | other | Keyboard settings sidebar | Example category for selecting keyboard shortcuts. |
| MC.09.01.e006 | App Shortcuts | other | Keyboard settings sidebar | Category for customizing shortcuts for specific apps. |
| MC.09.01.e007 | checkbox | control | Beside a shortcut in a Keyboard settings category | Select to enable or deselect to disable the shortcut. |
| MC.09.01.e008 | current key combination | control | Beside a shortcut in Keyboard settings | Double-click to replace the shortcut’s key combination. |
| MC.09.01.e009 | Keyboard Shortcuts | other | In the instructions for restoring defaults | The area to click in Keyboard settings before restoring defaults. |
| MC.09.01.e010 | Restore Defaults | button | Bottom-left corner of Keyboard Shortcuts | Returns all shortcuts to their original key combinations. |
| MC.09.01.e011 | File | menu | Menu bar example | Menu used in the example File > New Window. |
| MC.09.01.e012 | New Window | menu item | File menu example | Example menu item that can be invoked with Command-N. |
| MC.09.01.e013 | menu bar | view | Top of macOS apps | The page says shortcuts appear next to menu items here. |
| MC.09.01.e014 | Keyboard Viewer | other | Mentioned in the keyboard-layout note | Can show the current keyboard layout, known as an input source. |
| MC.09.01.e015 | input source | other | Mentioned in the keyboard-layout note | The page’s term for the current keyboard layout. |
| MC.09.01.e016 | Keyboard shortcuts and gestures in Safari | other | List of example app user guides | Linked guide listed as an example. |
| MC.09.01.e017 | Keyboard shortcuts in Preview | other | List of example app user guides | Linked guide listed as an example. |
| MC.09.01.e018 | Keyboard shortcuts in Disk Utility | other | List of example app user guides | Linked guide listed as an example. |
| MC.09.01.e019 | Mac keyboard shortcuts | other | Tip about finding shortcuts | Apple Support article referenced as a place to find shortcuts. |
| MC.09.01.e020 | Create keyboard shortcuts for apps | other | App Shortcuts instructions | Referenced article about creating app shortcuts. |
| MC.09.01.e021 | Change a conflicting keyboard shortcut | other | Shortcut conflict instructions | Referenced article about changing a conflicting shortcut. |
| MC.09.01.e022 | What are those symbols shown in menus on Mac? | other | See also | Related Apple Support article. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.09.01.b001 | Press and hold one or more modifier keys, then press the shortcut’s last key. | Performs the task associated with the keyboard shortcut. | The keys are pressed together; the page gives Command-V as an example. |
| MC.09.01.b002 | Press Command-N. | Opens a new window. | Given as the shortcut example for File > New Window. |
| MC.09.01.b003 | Press Command-V. | Pastes copied text. | Given as an example of using a keyboard shortcut. |
| MC.09.01.b004 | Look at menu items in macOS apps. | Keyboard shortcuts appear next to the menu items. | The page says many shortcuts are standard across apps. |
| MC.09.01.b005 | Use a shortcut that does not work as expected. | Check the app menus for the correct shortcut, or use Keyboard Viewer to see the current keyboard layout. | Shortcuts may vary with the language and keyboard layout used on the Mac. |
| MC.09.01.b006 | Select a category in the Keyboard settings sidebar. | Shows shortcuts for that category. | Examples given are Mission Control and Spotlight. |
| MC.09.01.b007 | Select App Shortcuts in the sidebar. | Allows customization of keyboard shortcuts for specific apps. | The page refers to “Create keyboard shortcuts for apps” for details. |
| MC.09.01.b008 | Select the checkbox next to a shortcut. | Enables or selects that shortcut. | In the customization instructions. |
| MC.09.01.b009 | Double-click the current key combination, then press a new key combination. | Changes the shortcut’s key combination. | Each type of key, such as a letter key or a function key, can be used only once in a key combination. |
| MC.09.01.b010 | Assign a shortcut that already exists for another command or app. | The new shortcut will not work; change the new shortcut or the other shortcut. | The page describes this as a shortcut conflict. |
| MC.09.01.b011 | Quit and reopen apps that are in use. | The new keyboard shortcut takes effect. | Required after changing a shortcut. |
| MC.09.01.b012 | In Keyboard settings, click Keyboard Shortcuts, then click Restore Defaults. | Returns all shortcuts to their original key combinations. | Restore Defaults is at the bottom-left corner. |
| MC.09.01.b013 | Deselect the checkbox next to a shortcut. | Disables the macOS keyboard shortcut. | The page presents this as a way to address a conflict between an app shortcut and a macOS shortcut. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.09.01.k001 | Command-N | Open a new window. | Example for File > New Window. |
| MC.09.01.k002 | Command-V | Paste copied text. | Example of pressing and holding Command while pressing V. |

System Settings paths referenced: Keyboard settings > Mission Control; Keyboard settings > Spotlight; Keyboard settings > App Shortcuts; Keyboard settings > Keyboard Shortcuts > Restore Defaults

#### Independent reports (MC.09.01)

| ID | Kind | Area | Claim | Source |
|---|---|---|---|---|
| MC.09.01.x01 | changed | shortcuts | Shortcuts on Mac has a three-pane interface with a shortcut preview that shows its color, icon, and an AI-generated summary. The preview pane can be resized or hidden. | T4 [Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/) |
| MC.09.01.x02 | new | shortcuts | A new "Otherwise If" branch can be added by control-clicking the Else block and choosing Add "Otherwise If". | T4 [Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/) |
| MC.09.01.x03 | behavior | shortcuts | The article says Sequoia's keyboard-driven tiling shortcuts are configured under System Settings → Desktop & Dock → Windows. | T4 [Appish](https://appish.app/blog/mac-window-tiling-shortcuts-not-working-fix) |
| MC.09.01.x04 | behavior | shortcuts | The article identifies conflicts with other apps' global shortcuts as a possible reason tiling shortcuts fail, citing Rectangle and Magnet's ⌃⌥← and ⌃⌥→ bindings as examples. | T4 [Appish](https://appish.app/blog/mac-window-tiling-shortcuts-not-working-fix) |
| MC.09.01.x05 | behavior | shortcuts | The article says Alfred, Raycast, or other apps may conflict with tiling shortcuts if configured to use the same modifier-heavy or global key combinations. | T4 [Appish](https://appish.app/blog/mac-window-tiling-shortcuts-not-working-fix) |
| MC.09.01.x06 | workaround | shortcuts | To check for conflicts, the article directs readers to System Settings → Keyboard → Keyboard Shortcuts → App Shortcuts and to the preferences of window managers or launchers. | T4 [Appish](https://appish.app/blog/mac-window-tiling-shortcuts-not-working-fix) |
| MC.09.01.x07 | behavior | shortcuts | The article says Sequoia's built-in tiling shortcuts cannot be remapped in System Settings → Keyboard; it recommends changing or disabling the conflicting app's shortcut instead. | T4 [Appish](https://appish.app/blog/mac-window-tiling-shortcuts-not-working-fix) |
| MC.09.01.x08 | behavior | shortcuts | The article says the keyboard-driven layout picker requires a mouse or trackpad that supports hover and will not appear using Magic Keyboard input alone. | T4 [Appish](https://appish.app/blog/mac-window-tiling-shortcuts-not-working-fix) |
| MC.09.01.x09 | behavior | shortcuts | The article says Stage Manager has no keyboard shortcut assigned by default, but one can be assigned in Keyboard Shortcuts under Mission Control. | T4 [Lets Suite](https://www.lets-suite.com/blog/how-to-use-stage-manager-mac) |
| MC.09.01.x10 | behavior | shortcuts | To assign a Stage Manager shortcut, tick its entry, double-click the shortcut area, and press the desired key combination. | T4 [Lets Suite](https://www.lets-suite.com/blog/how-to-use-stage-manager-mac) |
| MC.09.01.x11 | behavior | shortcuts | ⌘⇥ switches apps and moves to the set containing the selected app. | T4 [Lets Suite](https://www.lets-suite.com/blog/how-to-use-stage-manager-mac) |
| MC.09.01.x12 | behavior | shortcuts | ⌘` cycles through windows in the current set. | T4 [Lets Suite](https://www.lets-suite.com/blog/how-to-use-stage-manager-mac) |
| MC.09.01.x13 | behavior | shortcuts | The article says there are no shortcuts for adding a window to a Stage Manager set. | T4 [Lets Suite](https://www.lets-suite.com/blog/how-to-use-stage-manager-mac) |
| MC.09.01.x14 | behavior | shortcuts | The article says native tiling shortcuts remain Fn-Control chords and are still not remappable in “System Settings.” | T4 [Lets Suite](https://www.lets-suite.com/blog/macos-27-window-management) |
| MC.09.01.x15 | behavior | shortcuts | The article says no new native tiling shortcuts have been confirmed; tiling continues to use Fn/Globe-key chords introduced in Sequoia and carried through Tahoe. | T4 [Lets Suite](https://www.lets-suite.com/blog/macos-27-window-management) |
| MC.09.01.x16 | new | shortcuts | With Siri AI (Beta) enabled, Command-Shift-Space bar selects the active window for Visual Intelligence, and Command-Shift-6 lets the user select a screen area. | T4 [Pizazoo](https://pizazoo.com/blog/macos-27-screenshot-features/) |
| MC.09.01.x17 | behavior | shortcuts | Visual Intelligence can accept a question, offer suggested actions, or use Image Search to find similar images online. The article says this is for asking, acting, and searching, not saving a screenshot file. | T4 [Pizazoo](https://pizazoo.com/blog/macos-27-screenshot-features/) |
| MC.09.01.x18 | new | shortcuts | Shortcuts adds a Screenshot automation trigger named When a screenshot is saved, available on macOS as well as iOS and iPadOS. | T4 [Pizazoo](https://pizazoo.com/blog/macos-27-screenshot-features/) |
| MC.09.01.x19 | behavior | shortcuts | On Mac, users can open a shortcut, click Edit, then Automation, and choose a trigger. | T4 [Pizazoo](https://pizazoo.com/blog/macos-27-screenshot-features/) |
| MC.09.01.x20 | workaround | shortcuts | A suggested workflow uses Get Contents of Folder, Filter Files, and Move File to find the newest screenshot and move it to another folder. | T4 [Pizazoo](https://pizazoo.com/blog/macos-27-screenshot-features/) |
| MC.09.01.x21 | workaround | shortcuts | The suggested workflow can optionally use Show Notification to confirm that it ran. | T4 [Pizazoo](https://pizazoo.com/blog/macos-27-screenshot-features/) |
| MC.09.01.x22 | workaround | shortcuts | For renaming screenshots in the suggested workflow, the article recommends putting Rename File before Move File and using Format Date with time down to the second. | T4 [Pizazoo](https://pizazoo.com/blog/macos-27-screenshot-features/) |
| MC.09.01.x23 | new | shortcuts | Shortcuts adds the ability to extract tables or lists from an image, process a file directly with QR or barcode scanning, and append text to a specific section of a note. | T4 [Pizazoo](https://pizazoo.com/blog/macos-27-screenshot-features/) |
| MC.09.01.x24 | workaround | shortcuts | On Tahoe, the article identifies the Folder trigger as the closest alternative to the new screenshot trigger. | T4 [Pizazoo](https://pizazoo.com/blog/macos-27-screenshot-features/) |
| MC.09.01.x25 | workaround | shortcuts | The article says Apple does not specify whether the screenshot automation trigger fires for third-party capture apps and recommends testing with the app in use. | T4 [Pizazoo](https://pizazoo.com/blog/macos-27-screenshot-features/) |
| MC.09.01.x26 | new | shortcuts | Pressing Shift-Command-Space lets users select something onscreen for Visual Intelligence and ask Siri to explain it, translate text, search for similar images, or find related information. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MC.09.01.x27 | new | shortcuts | Shortcuts automations can now be triggered by taking a screenshot or receiving a notification. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MC.09.01.x28 | new | shortcuts | In Beta One, Bartender Command Bar supports mouse-free interaction for searching, activating presets, and viewing or running Automations; Bartender Pro also adds clipboard history. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.09.01.x29 | new | shortcuts | In Beta One, hotkeys include a Focus Mode that hides all menu bar items, and can also enable a preset. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.09.01.x30 | new | shortcuts | In Beta One, App Intents expose actions including getting items, showing or clicking them, changing profiles, and toggling the bar. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.09.01.x31 | new | shortcuts | In Beta Three, hotkeys can toggle Bartender Bar at the mouse location and quickly click an item to show it in the bar. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.09.01.x32 | new | shortcuts | Bartender 7 Beta Three adds AppleScript support for retrieving item details and app icons, and revealing or clicking items with left-click, right-click, and Option-click actions. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.09.01.x33 | fix | shortcuts | Bartender 7 Beta Three says exported app icons retain their correct colors in integrations such as Raycast and Shortcuts. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.09.01.x34 | new | shortcuts | Shortcuts can be created by typing or speaking a request instead of building them manually. | T4 [Tom's Guide](https://www.tomsguide.com/computing/macos/macos-golden-gate-review) |
| MC.09.01.x35 | new | shortcuts | Pressing Command-Shift-Space lets users select a region of the display and ask Siri about it. | T4 [9to5Mac](https://9to5mac.com/2026/09/14/macos-27-golden-gate-now-available-here-is-everything-new/) |
| MC.09.01.x36 | behavior | shortcuts | Press and hold Option while selecting “Date & Time” in the menu bar to turn on Do Not Disturb. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |

### MC.09.02 Trackpad gestures

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).

Also covered by: [Use Multi-Touch gestures on Mac](https://support.apple.com/en-us/102482)


#### View and customize mouse or trackpad gestures on Mac - Apple Support

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mh35869/27/mac/27) · macOS 27 text present: no

The page explains that gestures on a Magic Mouse or Apple trackpad can perform actions such as zooming in on PDFs, rotating photos, and showing the desktop. It describes how to view gesture demonstrations and customize or turn off available gestures in Mouse or Trackpad settings.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.09.02.e001 | Apple menu | menu | In the instruction above the settings links | The page says to choose this menu. |
| MC.09.02.e002 | Open Mouse settings for me | other | Below “Choose Apple menu” | A link that opens Mouse settings. |
| MC.09.02.e003 | Open Trackpad settings for me | other | Below “Choose Apple menu” | A link that opens Trackpad settings. |
| MC.09.02.e004 | Mouse settings | view | Referenced in the page text and link label | Settings where available mouse gestures can be viewed and customized. |
| MC.09.02.e005 | Trackpad settings | view | Referenced in the page text and link label | Settings where available trackpad gestures can be viewed and customized. |
| MC.09.02.e006 | click | gesture | In the gesture examples | Named as an example of a gesture. |
| MC.09.02.e007 | tap | gesture | In the gesture examples | Named as an example of a gesture. |
| MC.09.02.e008 | pinch | gesture | In the gesture examples | Named as an example of a gesture. |
| MC.09.02.e009 | Change Trackpad settings | other | In the sentence about gesture descriptions | A link to information about Trackpad settings. |
| MC.09.02.e010 | Mouse settings | other | In the sentence about gesture descriptions | A link to information about Mouse settings. |
| MC.09.02.e011 | Use Multi-Touch gestures on your Mac | other | See also | Apple Support article link. |
| MC.09.02.e012 | Change mouse or trackpad tracking, double-click, and scrolling speed on Mac | other | See also | Apple Support article link. |
| MC.09.02.e013 | Right-click on Mac | other | See also | Apple Support article link. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.09.02.b001 | Use gestures such as click, tap, and pinch with a Magic Mouse or Apple trackpad. | Gestures can be used to zoom in on PDFs, rotate photos, quickly show the desktop, and more. | The user is using a Magic Mouse or Apple trackpad with the Mac. The page does not map individual gestures to specific results. |
| MC.09.02.b002 | Open Mouse or Trackpad settings. | Available gestures can be browsed, and an animation demonstrating each gesture can be viewed. | A wireless mouse or trackpad must be connected to the Mac, or the Mac must have a built-in trackpad. |
| MC.09.02.b003 | Customize a gesture in Mouse or Trackpad settings. | The gesture can be customized or turned off. | A wireless mouse or trackpad must be connected to the Mac, or the Mac must have a built-in trackpad. |
| MC.09.02.b004 | Choose “Open Mouse settings for me.” | Mouse settings open. | — |
| MC.09.02.b005 | Choose “Open Trackpad settings for me.” | Trackpad settings open. | — |
| MC.09.02.b006 | Modify the control referenced by “Modifying this control will update this page automatically.” | The page updates automatically. | The page does not identify the control in the supplied text. |

System Settings paths referenced: Mouse settings; Trackpad settings

#### Independent reports (MC.09.02)

| ID | Kind | Area | Claim | Source |
|---|---|---|---|---|
| MC.09.02.x01 | new | gestures | Pull-to-refresh is available in Safari, Mail, News, Podcasts, and Calendar. | T3 [Wikipedia](https://en.wikipedia.org/wiki/MacOS_Golden_Gate) |
| MC.09.02.x02 | behavior | gestures | In Notes, you can draw by clicking and dragging on a Magic Trackpad, or by holding down the mouse button and dragging. The same drawing feature is available in Freeform. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/12/) |
| MC.09.02.x03 | behavior | gestures | A commenter says they use trackpad gestures to navigate between applications that are mostly in full-screen mode. | T5 [Reddit r/MacOS](https://www.reddit.com/r/MacOS/comments/1wlax16/mission_control_expose_is_broken_in_macos_27/) |
| MC.09.02.x04 | new | gestures | Sidecar supports iPad touchscreen input for interacting with the Mac, including scrolling Mac browser windows and dragging windows. The reviewer also reports being able to scroll a Finder window with a finger. | T4 [Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/) |
| MC.09.02.x05 | behavior | gestures | With Sidecar touchscreen support, menus drop down reliably, buttons are tappable, and scrolling works. | T4 [Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/) |
| MC.09.02.x06 | new | gestures | Safari, Mail, News, and other first-party apps will support swipe down to refresh. | T4 [Cult of Mac](https://www.cultofmac.com/news/macos-27-new-features-beyond-siri-ai) |
| MC.09.02.x07 | new | gestures | A swipe-down-to-refresh gesture is available in Safari, Mail, News, Podcasts, and Calendar, using a trackpad or a finger on iPad via Sidecar. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MC.09.02.x08 | new | gestures | Sidecar lets users interact directly with the Mac interface on a compatible iPad used as a second display, including tapping controls, scrolling with a finger, and using more system gestures. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MC.09.02.x09 | new | gestures | AppKit adds NSRefreshController to NSScrollView for pull-to-refresh; refresh can be initiated by the user or programmatically and ended when complete. | T5 [MacRumors Forums](https://forums.macrumors.com/threads/macos-27-all-the-little-things.2483520/) |
| MC.09.02.x10 | new | gestures | The NSApplication presentation options add an option developers can use to disable Hot Corners. | T5 [MacRumors Forums](https://forums.macrumors.com/threads/macos-27-all-the-little-things.2483520/) |
| MC.09.02.x11 | new | gestures | The article lists pull to refresh in Safari on macOS as an upcoming small improvement. | T4 [Gizmodo](https://gizmodo.com/im-more-excited-for-the-tiny-changes-in-apples-27-software-updates-than-siri-ai-tbh-2000769815) |
| MC.09.02.x12 | behavior | gestures | In Photos’ Reframe tool, swiping a finger across the trackpad changes the photo’s perspective. | T4 [Tom's Guide](https://www.tomsguide.com/computing/macos/macos-golden-gate-review) |
| MC.09.02.x13 | api | gestures | AppKit adds NSRefreshController to provide pull-to-refresh functionality for NSScrollView. It can handle user-initiated refreshes and start refreshing programmatically. | T4 [BetaProfile](https://betaprofile.dev/macos-27-rc/) |
| MC.09.02.x14 | new | gestures | Safari supports a pull-to-refresh gesture: users swipe down past the rubber-band effect to refresh a page. The gesture is also available in Mail, News, and Calendar. | T4 [9to5Mac](https://9to5mac.com/2026/09/14/macos-27-golden-gate-now-available-here-is-everything-new/) |
| MC.09.02.x15 | behavior | gestures | Swipe left from the right edge of the trackpad with two fingers to open Notification Center; swipe right with two fingers to close it. The gesture requires “Notification Center” to be enabled in Trackpad settings. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |
| MC.09.02.x16 | behavior | gestures | The trackpad gesture setting is at System Settings > Trackpad > More Gestures. Toggle on “Notification Center” to enable the gesture. | T4 [Switching to Mac](https://www.switchingtomac.com/notification-center-on-mac-how-to-customize-and-use-it/) |

### MC.09.03 Full Keyboard Access

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).

Also covered by: [Mac keyboard shortcuts - Apple Support](https://support.apple.com/en-us/102650)


#### Use your keyboard like a mouse with Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mh27469/27/mac/27) · macOS 27 text present: no

Mouse Keys lets you move the pointer and perform mouse-button actions using keyboard keys. The page also references Full Keyboard Access, Accessibility Shortcuts, and VoiceOver.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.09.03.e001 | Apple menu | menu | Referenced in the instructions for opening Pointer Control settings | Menu to choose when opening Pointer Control settings. |
| MC.09.03.e002 | Open Pointer Control settings for me | other | Link in the instructions for turning Mouse Keys on or off | Opens Pointer Control settings. |
| MC.09.03.e003 | Pointer Control settings | panel | Referenced in the Mouse Keys instructions | Settings where Mouse Keys can be turned on or off. |
| MC.09.03.e004 | Mouse Keys | setting | Pointer Control settings | When on, lets you control the pointer and mouse button with keyboard keys. |
| MC.09.03.e005 | Accessibility Shortcuts | other | Tip in the Mouse Keys instructions | Can be used to quickly turn Mouse Keys on or off. |
| MC.09.03.e006 | Full Keyboard Access | setting | Tip near the beginning of the page | Referenced as a way to navigate between onscreen items with keyboard shortcuts. |
| MC.09.03.e007 | VoiceOver | other | Final paragraph | Referenced as a way to hear what is on screen and navigate and interact with it using keyboard commands. |
| MC.09.03.e008 | VoiceOver User Guide | other | Final paragraph | Referenced for more information about VoiceOver. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.09.03.b001 | Turn Mouse Keys on in Pointer Control settings | Use the keyboard to move the pointer and click the mouse. | Mouse Keys must be on. |
| MC.09.03.b002 | Turn Mouse Keys off in Pointer Control settings | Mouse Keys is off. | The setting is in Pointer Control settings. |
| MC.09.03.b003 | Use the keyboard or numeric keypad while Mouse Keys is on | You cannot use it to enter text. | Mouse Keys is on. |
| MC.09.03.b004 | Use Accessibility Shortcuts | Quickly turn Mouse Keys on or off. | No additional conditions stated. |
| MC.09.03.b005 | Use Full Keyboard Access keyboard shortcuts | Navigate between onscreen items. | Full Keyboard Access is being used. |
| MC.09.03.b006 | Press the listed movement keys | Move the mouse pointer. | Mouse Keys is on; use the numeric keypad keys 7, 8, 9, 4, 6, 1, 2, 3, or the keyboard keys 7, 8, 9, U, O, J, K, L. |
| MC.09.03.b007 | Press 5 on the numeric keypad or I on the keyboard | Click a mouse button. | Mouse Keys is on. |
| MC.09.03.b008 | Press 0 on the numeric keypad or M on the keyboard | Press and hold the mouse button. | Mouse Keys is on. |
| MC.09.03.b009 | Press the period key | Release the mouse button. | Mouse Keys is on; the period can be on the numeric keypad or keyboard. |
| MC.09.03.b010 | Use keyboard shortcuts, assistive switches, or facial expressions | Perform mouse actions, such as left, right, or double clicks. | No additional conditions stated. |
| MC.09.03.b011 | Use VoiceOver keyboard commands | Hear what is on the screen and navigate and interact with it. | The page identifies this for people who are blind or have low vision. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.09.03.k001 | Numeric keypad: 7, 8, 9, 4, 6, 1, 2, 3 | Move the mouse pointer. | Mouse Keys is on. |
| MC.09.03.k002 | Keyboard: 7, 8, 9, U, O, J, K, L | Move the mouse pointer. | Mouse Keys is on. |
| MC.09.03.k003 | Numeric keypad: 5 | Click a mouse button. | Mouse Keys is on. |
| MC.09.03.k004 | I | Click a mouse button. | Keyboard with Mouse Keys on. |
| MC.09.03.k005 | Numeric keypad: 0 | Press and hold the mouse button. | Mouse Keys is on. |
| MC.09.03.k006 | M | Press and hold the mouse button. | Keyboard with Mouse Keys on. |
| MC.09.03.k007 | Period (.) | Release the mouse button. | Mouse Keys is on; usable on the numeric keypad or keyboard. |

System Settings paths referenced: Pointer Control settings

#### Navigate your Mac using Full Keyboard Access

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlc06d1059/27/mac/27) · macOS 27 text present: no

Full Keyboard Access lets you navigate and interact with your Mac using the keyboard. You can customize its highlight and commands, or temporarily pause it with Pass-Through Mode.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.09.03.e009 | Apple menu | menu | Menu bar | Starting point for the instructions to open Keyboard settings for accessibility. |
| MC.09.03.e010 | Motor | other | System Settings path in the instructions | The page says to go to Motor, then click Keyboard. |
| MC.09.03.e011 | Keyboard | settings | Under Motor in the stated settings path | The page directs users to Keyboard settings to turn on or off and customize Full Keyboard Access. |
| MC.09.03.e012 | Open Keyboard settings for accessibility for me | button | Within the instructions for opening Keyboard settings | A link or button label shown as an option to open Keyboard settings for accessibility. |
| MC.09.03.e013 | Full Keyboard Access | setting | Keyboard settings | Can be turned on or off. |
| MC.09.03.e014 | Full Keyboard Access highlight | other | On-screen focus indication | Highlights the item on the screen that has focus. Its color, contrast, and size can be customized. |
| MC.09.03.e015 | Commands Help | window | Opened with Tab-H when Full Keyboard Access is on | Shows Full Keyboard Access commands. |
| MC.09.03.e016 | Full Keyboard Access settings | settings | Keyboard settings | The page says a complete list of commands is available here. |
| MC.09.03.e017 | Commands | view | Full Keyboard Access customization settings | The section where a command can be selected to customize its shortcut. |
| MC.09.03.e018 | Restore Defaults | button | Bottom of the Commands list | Restores default keyboard shortcuts and removes shortcuts for advanced commands, if any. |
| MC.09.03.e019 | Done | button | Full Keyboard Access customization settings | The page says to click it after making customizations. |
| MC.09.03.e020 | Pass-Through Mode | setting | Full Keyboard Access | Temporarily turns Full Keyboard Access off; it can be turned back on. |
| MC.09.03.e021 | VoiceOver | other | See also text | The page says VoiceOver keyboard commands can be used to hear what's on the screen and navigate and interact with it. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.09.03.b012 | Turn on Full Keyboard Access | The keyboard can be used to navigate the Mac instead of a mouse or trackpad, and the focused item is highlighted. | Full Keyboard Access is on. |
| MC.09.03.b013 | Turn Full Keyboard Access on or off in Keyboard settings | Enables or disables Full Keyboard Access. | Follow the stated Apple menu > Motor > Keyboard path. |
| MC.09.03.b014 | Press Tab | Moves to the next interface element. | Full Keyboard Access is on; default command. |
| MC.09.03.b015 | Press Shift-Tab | Moves to the previous interface element. | Full Keyboard Access is on; default command. |
| MC.09.03.b016 | Press the Space bar | Selects an item. | Full Keyboard Access is on; default command. |
| MC.09.03.b017 | Press an Up Arrow, Down Arrow, Left Arrow, or Right Arrow key | Moves inside a group, such as a list of files in a folder or items in a sidebar. Full Keyboard Access highlights the group and the item within the group that has focus. | Full Keyboard Access is on; default command. |
| MC.09.03.b018 | Press Control-Tab | Moves to the next item. | Full Keyboard Access is on; default command. |
| MC.09.03.b019 | Press Control-Shift-Tab | Moves to the previous item. | Full Keyboard Access is on; default command. |
| MC.09.03.b020 | Press Tab-F | Searches for items. | Full Keyboard Access is on; default command. |
| MC.09.03.b021 | Press Tab-W | Opens the Window Chooser. | Full Keyboard Access is on; default command. |
| MC.09.03.b022 | Press Tab-A | Opens the Application Chooser. | Full Keyboard Access is on; default command. |
| MC.09.03.b023 | Press Fn-Control-F2 | Jumps to the menu bar. | Full Keyboard Access is on; default command. |
| MC.09.03.b024 | Press Fn-A | Jumps to the Dock. | Full Keyboard Access is on; default command. |
| MC.09.03.b025 | Press Fn-C | Opens Control Center. | Full Keyboard Access is on; default command. |
| MC.09.03.b026 | Press Fn-N | Opens Notification Center. | Full Keyboard Access is on; default command. |
| MC.09.03.b027 | Press Tab-C | Moves the pointer to where the keyboard focus is. | Full Keyboard Access is on; default command. |
| MC.09.03.b028 | Press Tab-H | Opens Commands Help, where Full Keyboard Access commands can be viewed. | Full Keyboard Access is on. |
| MC.09.03.b029 | Choose options for the appearance of the Full Keyboard Access highlight | Changes the highlight's appearance, including its color, size, and more. | In Full Keyboard Access customization settings. |
| MC.09.03.b030 | Select a command in Commands and press the desired key or keys | Sets a custom keyboard shortcut for that command. | In Full Keyboard Access customization settings. |
| MC.09.03.b031 | Click Restore Defaults | Restores default keyboard shortcuts and removes shortcuts for advanced commands, if any. | At the bottom of the Commands list. |
| MC.09.03.b032 | Click Done | Finishes the customization workflow. | After making customizations. |
| MC.09.03.b033 | Press Control-Option-Command-P | Turns Pass-Through Mode on or off, temporarily turning Full Keyboard Access off or back on. | Full Keyboard Access is on. |
| MC.09.03.b034 | Use Tab or Shift-Tab | Navigates to the next or previous control on the screen, respectively. | When using the option described for people who do not need the complete set of Full Keyboard Access commands. |
| MC.09.03.b035 | Use VoiceOver keyboard commands | Hears what's on the screen and navigates and interacts with it. | The page presents this as an option for people who are blind or have low vision. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.09.03.k008 | Tab | Move to the next interface element. | Full Keyboard Access on; default command. |
| MC.09.03.k009 | Shift-Tab | Move to the previous interface element. | Full Keyboard Access on; default command. |
| MC.09.03.k010 | Space bar | Select an item. | Full Keyboard Access on; default command. |
| MC.09.03.k011 | Up Arrow, Down Arrow, Left Arrow, or Right Arrow | Move inside a group. | Full Keyboard Access on; default command. |
| MC.09.03.k012 | Control-Tab | Move to the next item. | Full Keyboard Access on; default command. |
| MC.09.03.k013 | Control-Shift-Tab | Move to the previous item. | Full Keyboard Access on; default command. |
| MC.09.03.k014 | Tab-F | Search for items. | Full Keyboard Access on; default command. |
| MC.09.03.k015 | Tab-W | Open the Window Chooser. | Full Keyboard Access on; default command. |
| MC.09.03.k016 | Tab-A | Open the Application Chooser. | Full Keyboard Access on; default command. |
| MC.09.03.k017 | Fn-Control-F2 | Jump to the menu bar. | Full Keyboard Access on; default command. |
| MC.09.03.k018 | Fn-A | Jump to the Dock. | Full Keyboard Access on; default command. |
| MC.09.03.k019 | Fn-C | Open Control Center. | Full Keyboard Access on; default command. |
| MC.09.03.k020 | Fn-N | Open Notification Center. | Full Keyboard Access on; default command. |
| MC.09.03.k021 | Tab-C | Move the pointer to where the keyboard focus is. | Full Keyboard Access on; default command. |
| MC.09.03.k022 | Tab-H | Open Commands Help. | Full Keyboard Access on. |
| MC.09.03.k023 | Control-Option-Command-P | Turn Pass-Through Mode on or off. | Full Keyboard Access on. |
| MC.09.03.k024 | Tab | Navigate to the next control on the screen. | Option described for users who do not need the complete set of Full Keyboard Access commands. |
| MC.09.03.k025 | Shift-Tab | Navigate to the previous control on the screen. | Option described for users who do not need the complete set of Full Keyboard Access commands. |

System Settings paths referenced: Apple menu > Motor > Keyboard; Full Keyboard Access settings

#### Independent reports (MC.09.03)

| ID | Kind | Area | Claim | Source |
|---|---|---|---|---|
| MC.09.03.x01 | behavior | keyboard navigation | “Cmd-Tab” switches between apps, not individual windows; multiple Chrome windows are still represented as one app. | T4 [Lets Suite](https://www.lets-suite.com/blog/macos-27-window-management) |
| MC.09.03.x02 | behavior | keyboard navigation | “Cmd-`” cycles through windows of the same app only within the current Space, not across desktops. | T4 [Lets Suite](https://www.lets-suite.com/blog/macos-27-window-management) |
| MC.09.03.x03 | workaround | keyboard navigation | The article says Let's Switch uses “Option-Tab” to show live previews of open windows across monitors, most recent first, and “Option-backtick” to cycle through one app's windows. It says Pro adds an “Option-Space” search palette. | T4 [Lets Suite](https://www.lets-suite.com/blog/macos-27-window-management) |
| MC.09.03.x04 | behavior | keyboard navigation | By default, keyboard navigation on macOS is limited. Enabling the setting allows the Tab key to focus controls in pop-up dialogs and Preferences windows. | T4 [tempertemper](https://www.tempertemper.net/blog/using-the-keyboard-to-navigate-on-macos) |
| MC.09.03.x05 | behavior | keyboard navigation | With keyboard navigation enabled, focus can move to text input and search boxes, dropdown menus, range sliders, radio buttons, checkboxes, number steppers, tab bar buttons, icon-style buttons, text-only buttons, and fully coloured buttons. | T4 [tempertemper](https://www.tempertemper.net/blog/using-the-keyboard-to-navigate-on-macos) |
| MC.09.03.x06 | behavior | keyboard navigation | The setting’s label explains that Tab moves focus forward and Shift-Tab moves focus backward. | T4 [tempertemper](https://www.tempertemper.net/blog/using-the-keyboard-to-navigate-on-macos) |
| MC.09.03.x07 | behavior | keyboard navigation | When a text or search box has focus, type to enter text. | T4 [tempertemper](https://www.tempertemper.net/blog/using-the-keyboard-to-navigate-on-macos) |
| MC.09.03.x08 | behavior | keyboard navigation | With a dropdown menu focused, the Up and Down arrow keys or Space activate the menu and cycle through its options; Space or Return selects an option. | T4 [tempertemper](https://www.tempertemper.net/blog/using-the-keyboard-to-navigate-on-macos) |
| MC.09.03.x09 | behavior | keyboard navigation | With a range slider focused, Left and Right move the slider. | T4 [tempertemper](https://www.tempertemper.net/blog/using-the-keyboard-to-navigate-on-macos) |
| MC.09.03.x10 | behavior | keyboard navigation | In a radio group, the Left and Right or Up and Down arrow keys move the selected option. | T4 [tempertemper](https://www.tempertemper.net/blog/using-the-keyboard-to-navigate-on-macos) |
| MC.09.03.x11 | behavior | keyboard navigation | Space toggles a focused checkbox on or off. | T4 [tempertemper](https://www.tempertemper.net/blog/using-the-keyboard-to-navigate-on-macos) |
| MC.09.03.x12 | behavior | keyboard navigation | Up and Down increase or decrease the value in a focused number stepper. | T4 [tempertemper](https://www.tempertemper.net/blog/using-the-keyboard-to-navigate-on-macos) |
| MC.09.03.x13 | behavior | keyboard navigation | Left and Right cycle between tabs. | T4 [tempertemper](https://www.tempertemper.net/blog/using-the-keyboard-to-navigate-on-macos) |
| MC.09.03.x14 | behavior | keyboard navigation | Space activates a focused icon-style or text-only button. | T4 [tempertemper](https://www.tempertemper.net/blog/using-the-keyboard-to-navigate-on-macos) |
| MC.09.03.x15 | behavior | keyboard navigation | Full Keyboard Access is enabled in System Settings → Accessibility → Keyboard, under the Motor section. | T4 [Lets Suite](https://www.lets-suite.com/blog/full-keyboard-access-mac) |
| MC.09.03.x16 | behavior | keyboard navigation | With Full Keyboard Access enabled, a focus ring highlights one element at a time. Tab moves focus forward, Shift-Tab moves it backward, arrow keys move within a group, and Space activates the focused control. | T4 [Lets Suite](https://www.lets-suite.com/blog/full-keyboard-access-mac) |
| MC.09.03.x17 | behavior | keyboard navigation | Tab-F searches the current window's controls, and Tab-C moves the pointer to the focused item. | T4 [Lets Suite](https://www.lets-suite.com/blog/full-keyboard-access-mac) |
| MC.09.03.x18 | behavior | keyboard navigation | Tab-H opens Commands Help, described as a searchable list of Full Keyboard Access commands. | T4 [Lets Suite](https://www.lets-suite.com/blog/full-keyboard-access-mac) |
| MC.09.03.x19 | behavior | keyboard navigation | The focus ring's visibility, color, and size can be customized using the ⓘ button beside the Full Keyboard Access toggle. | T4 [Lets Suite](https://www.lets-suite.com/blog/full-keyboard-access-mac) |
| MC.09.03.x20 | behavior | keyboard navigation | The separate Keyboard navigation toggle is in System Settings → Keyboard; it lets Tab move between controls in dialogs and web forms. | T4 [Lets Suite](https://www.lets-suite.com/blog/full-keyboard-access-mac) |
| MC.09.03.x21 | behavior | keyboard navigation | The article contrasts Keyboard navigation's dialog-and-form focus with Full Keyboard Access, which reaches controls throughout the interface and displays a more prominent focus ring. | T4 [Lets Suite](https://www.lets-suite.com/blog/full-keyboard-access-mac) |
| MC.09.03.x22 | bug | keyboard navigation | Fixed an issue where keyboard focus and tab order in expanded or collapsed open and save panels might not traverse all fields and controls, including the accessory view, in the expected order. | T4 [BetaProfile](https://betaprofile.dev/macos-27-rc/) |
| MC.09.03.x23 | bug | keyboard navigation | Fixed an issue where, with full keyboard navigation off, the keyboard loop might not traverse the correct fields and controls. | T4 [BetaProfile](https://betaprofile.dev/macos-27-rc/) |

### MC.09.04 Screenshots (Cmd-Shift-3/4/5)

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).


#### Take screenshots or screen recordings on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mh26782/27/mac/27) · macOS 27 text present: no

Use Screenshot or keyboard shortcuts to capture images or record the screen. Screenshot offers capture modes and options for saving, timing, microphone input, floating thumbnails, selection memory, capture format, and pointer visibility.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.09.04.e001 | Screenshot | app | Opened with Shift-Command-5 or Spotlight | Provides tools for taking screenshots and screen recordings. |
| MC.09.04.e002 | Capture the entire screen | tool | Screenshot tools | Selects the entire screen for a screenshot. |
| MC.09.04.e003 | Capture a window | tool | Screenshot tools | Selects a window for a screenshot. |
| MC.09.04.e004 | Capture a portion of the screen | tool | Screenshot tools | Selects a portion of the screen for a screenshot. |
| MC.09.04.e005 | Record the entire screen | tool | Screenshot tools | Selects the entire screen for a recording. |
| MC.09.04.e006 | Record a window | tool | Screenshot tools | Selects a window for a recording. |
| MC.09.04.e007 | Record a portion of the screen | tool | Screenshot tools | Selects a portion of the screen for a recording. |
| MC.09.04.e008 | Options | menu | Screenshot tools | Pop-up menu containing capture and recording options; available options vary by capture type. |
| MC.09.04.e009 | Save to | setting | Options | Offers preset save locations and the Other Location option. |
| MC.09.04.e010 | Other Location | menu item | Options > Save to | Opens navigation to choose a folder. |
| MC.09.04.e011 | Timer | setting | Options | Sets how long to wait before the screen capture begins. |
| MC.09.04.e012 | Microphone | setting | Options | Selects a microphone for a screen recording. |
| MC.09.04.e013 | None | menu item | Options > Microphone | Turns off the microphone. |
| MC.09.04.e014 | Show Floating Thumbnail | setting | Options | Shows or hides the thumbnail after a capture. |
| MC.09.04.e015 | Remember Last Selection | setting | Options | Controls whether a selected screen portion is reused for the next capture. |
| MC.09.04.e016 | Capture Format | setting | Options | Opens the format choices HDR and SDR. |
| MC.09.04.e017 | HDR | menu item | Options > Capture Format | Captures more brightness and color; an HDR display is required to see the difference. |
| MC.09.04.e018 | SDR | menu item | Options > Capture Format | Captures less brightness and color and is standard for most computer displays and mobile devices. |
| MC.09.04.e019 | Show Mouse Pointer | setting | Options | Controls whether the mouse pointer is included in the picture. |
| MC.09.04.e020 | Capture | button | Screenshot tools | Starts a screenshot of the entire screen or a selected portion. |
| MC.09.04.e021 | Record | button | Screenshot tools | Starts a screen recording. |
| MC.09.04.e022 | Floating thumbnail | thumbnail | Bottom-right corner of the screen | Appears briefly after a completed screenshot or recording when Show Floating Thumbnail is enabled. |
| MC.09.04.e023 | Crosshair pointer | pointer | After Shift-Command-4 | Used to choose the starting point for a portion screenshot. |
| MC.09.04.e024 | Camera pointer | pointer | After Shift-Command-4, then Space bar | Highlights a window or menu bar item for capture. |
| MC.09.04.e025 | Keyboard settings | setting | Settings | Referenced as the place to customize keyboard shortcuts. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.09.04.b001 | Press Shift-Command-5 or use Spotlight. | Screenshot opens and displays its tools. | On Mac. |
| MC.09.04.b002 | Click a Screenshot tool. | Selects what to capture or record. | Choose one of the six screen, window, or portion tools. |
| MC.09.04.b003 | Drag the frame for a selected screen portion. | Repositions the frame; dragging its edges changes the area size. | When capturing or recording a portion of the screen. |
| MC.09.04.b004 | Click Options and choose an option. | Sets available capture or recording options. | Available options vary depending on whether taking a screenshot or recording. |
| MC.09.04.b005 | Choose a preset location below Save to, or click Other Location and navigate to a folder. | Specifies where the file will be saved. | In Options. |
| MC.09.04.b006 | Choose a time below Timer. | Sets the wait before the screen capture begins. | In Options. |
| MC.09.04.b007 | Select a microphone below Microphone. | A checkmark appears next to the selected microphone and it is used during the recording. | For a screen recording. |
| MC.09.04.b008 | Select None below Microphone. | Turns off the microphone. | For a screen recording. |
| MC.09.04.b009 | Select Show Floating Thumbnail in Options. | A checkmark appears and the thumbnail is shown briefly in the bottom-right corner after a completed capture. | The thumbnail allows time to drag, mark up, or share the capture before it is saved to the specified location. |
| MC.09.04.b010 | Select Show Floating Thumbnail when it is checked. | The checkmark disappears and the floating thumbnail is hidden. | In Options. |
| MC.09.04.b011 | Swipe right on the floating thumbnail. | Immediately saves the file and makes the thumbnail disappear. | While the thumbnail is briefly displayed. |
| MC.09.04.b012 | Drag the floating thumbnail into a document, email, note, or Finder window. | Places the capture in that destination. | While the thumbnail is briefly displayed. |
| MC.09.04.b013 | Click the floating thumbnail. | Opens a window to mark up a screenshot, trim a recording, or share it. | While the thumbnail is briefly displayed. |
| MC.09.04.b014 | Select Remember Last Selection in Options. | A checkmark appears and the selected portion is saved for the next screen capture. | When a portion of the screen is selected. |
| MC.09.04.b015 | Select Remember Last Selection when it is checked. | The checkmark disappears and the last selection is not saved. | In Options. |
| MC.09.04.b016 | Select HDR below Capture Format. | Captures more brightness and color. | An HDR display is required to see the difference. |
| MC.09.04.b017 | Show an HDR screenshot on an SDR display. | The screenshot appears in SDR format. | The display is SDR. |
| MC.09.04.b018 | Show an HDR screen recording on an SDR display. | The recording appears with slightly reduced quality. | The display is SDR. |
| MC.09.04.b019 | Select Show Mouse Pointer in Options. | A checkmark appears and the mouse pointer is included in the picture. | For a screenshot. |
| MC.09.04.b020 | Select Show Mouse Pointer when it is checked. | The checkmark disappears and the mouse pointer is not included in the picture. | For a screenshot. |
| MC.09.04.b021 | Click Capture. | Takes the screenshot. | For the entire screen or a selected portion. |
| MC.09.04.b022 | Move the pointer to a window and click the window. | Captures the window. | For a window capture. |
| MC.09.04.b023 | Click Record. | Starts the screen recording. | For a recording. |
| MC.09.04.b024 | Click the stop control. | Stops the recording. | The article does not provide a label for the control. |
| MC.09.04.b025 | Use a keyboard shortcut to take a screenshot. | Saves the screenshot file to the desktop. | Unless another destination is specified using Screenshot options. |
| MC.09.04.b026 | Hold Control while pressing the screenshot shortcut keys. | Copies the screenshot so it can be pasted elsewhere. | For example, Control-Shift-Command-3 copies the whole screen. |
| MC.09.04.b027 | Press Shift-Command-4, then press the Space bar; move the camera pointer over a window or the menu bar and click. | Captures the highlighted window or menu bar. | Keyboard shortcut method. |
| MC.09.04.b028 | Open a menu, press Shift-Command-4, then drag the pointer over menu items. | Captures the selected menu and menu items. | Keyboard shortcut method. |
| MC.09.04.b029 | Customize keyboard shortcuts in Keyboard settings. | Changes keyboard shortcuts. | On Mac. |
| MC.09.04.b030 | Take a screenshot or screen recording. | Screenshots are saved as .png files and recordings as .mov files; filenames begin with “Screenshot” or “Screen Recording” and include the date and time. | The file is saved according to the selected save location. |
| MC.09.04.b031 | Take a picture of a window in an app such as DVD Player. | The picture may not be available. | Some apps do not allow pictures of their windows. |
| MC.09.04.b032 | Complete a screenshot or recording. | An app may open. | Depends on where the screenshot or recording was saved. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.09.04.k001 | Shift-Command-5 | Open Screenshot. | Displays the Screenshot tools. |
| MC.09.04.k002 | Shift-Command-3 | Capture the entire screen. | Screenshot is saved to the desktop. |
| MC.09.04.k003 | Shift-Command-4 | Begin capturing a portion of the screen. | Move the crosshair to the starting point, drag over the area, then release the mouse or trackpad button. |
| MC.09.04.k004 | Shift-Command-4, then Space bar | Capture a window or the menu bar. | Move the camera pointer over the target to highlight it, then click. |
| MC.09.04.k005 | Shift-Command-4 | Capture a menu and menu items. | Open the menu first, then drag the pointer over the items to capture. |
| MC.09.04.k006 | Shift-Command-6 | Capture the Touch Bar. | Keyboard shortcut method. |
| MC.09.04.k007 | Control-Shift-Command-3 | Copy a screenshot of the entire screen. | Hold Control while pressing the screenshot shortcut keys. |

System Settings paths referenced: Keyboard settings

#### How to record the screen on Mac

T1 · OS documentation · [Apple Support](https://support.apple.com/en-us/102618) · macOS 27 text present: yes

Record the entire screen or a selected area using the Screenshot app or QuickTime Player. Available controls and recording options depend on the macOS version, Mac model, and which app performs the recording.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.09.04.e026 | Screenshot | app | Opened with Shift-Command-5 | App used to record the screen through the onscreen toolbar. |
| MC.09.04.e027 | onscreen toolbar | toolbar | On screen after opening Screenshot | Contains controls for capturing still images, followed by controls for recording video. |
| MC.09.04.e028 | Record Entire Screen | control | Screenshot toolbar | Control for recording the entire screen. |
| MC.09.04.e029 | Record Selected Window | control | Screenshot toolbar | Control for recording a selected window; available in macOS Tahoe 26 or later. |
| MC.09.04.e030 | Record Selected Portion | control | Screenshot toolbar | Control for recording a selected portion of the screen. |
| MC.09.04.e031 | borders | control | Around the selected portion | Can be dragged with a mouse or trackpad to adjust the selected portion. |
| MC.09.04.e032 | Options | button | Screenshot toolbar | Opens recording settings before recording starts. |
| MC.09.04.e033 | microphone | setting | Recording options | Choose a microphone to record your voice or other audio. |
| MC.09.04.e034 | Include System Audio | setting | Screenshot recording options | Option to record system audio; available in macOS 27 Golden Gate or later. |
| MC.09.04.e035 | Show Mouse Clicks | setting | Screenshot recording options | Shows a black circle around the pointer when you click. |
| MC.09.04.e036 | recording timer | setting | Screenshot recording options | Choose the number of seconds to wait after clicking Record before recording begins. |
| MC.09.04.e037 | Save to | setting | Screenshot recording options | Sets where the recording is saved after recording stops; recordings are saved to the desktop by default. |
| MC.09.04.e038 | SDR (Most Compatible) | setting | Screenshot recording options | Captures standard dynamic range using the H.264 video format; available on supported Mac models using macOS Tahoe 26 or later. |
| MC.09.04.e039 | HDR | setting | Screenshot recording options | Captures high dynamic range using the HEVC video format; available on supported Mac models using macOS Tahoe 26 or later. |
| MC.09.04.e040 | Record | button | Screenshot toolbar or QuickTime Player recording controls | Starts a screen recording. |
| MC.09.04.e041 | Stop | button | Menu bar during recording | Stops a screen recording. |
| MC.09.04.e042 | thumbnail | view | Corner of the screen after recording stops | Click to edit the recording; otherwise, the recording saves to the desktop. |
| MC.09.04.e043 | QuickTime Player | app | Applications folder | Can be used to record the screen and to play, edit, or share a recording. |
| MC.09.04.e044 | File | menu | Menu bar in QuickTime Player | Contains the New Screen Recording command. |
| MC.09.04.e045 | New Screen Recording | menu item | QuickTime Player > File | Opens the screen-recording controls. |
| MC.09.04.e046 | Screen Recording window | window | QuickTime Player | Indicates that screen recording is being performed by QuickTime Player. |
| MC.09.04.e047 | arrow next to the Record button | control | QuickTime Player Screen Recording window | Opens recording settings. |
| MC.09.04.e048 | volume slider | control | QuickTime Player recording settings | Adjusts audio monitoring volume during recording. |
| MC.09.04.e049 | Show Mouse Clicks in Recording | setting | QuickTime Player recording settings | Shows a black circle around the pointer when you click. |
| MC.09.04.e050 | Start Recording | button | Within a selected area in QuickTime Player | Starts recording the selected area. |
| MC.09.04.e051 | Screen Recording [date] at [time].mov | other | Automatically saved recording name | Name format used when saving a recording automatically. |
| MC.09.04.e052 | Apple TV app | app | Not specified | Example of an app that might not let you record its windows. |
| MC.09.04.e053 | iMovie | app | Not specified | Can open screen recordings. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.09.04.b033 | Press Shift-Command-5 | Opens the Screenshot app and displays its onscreen toolbar. | Using the Screenshot app. |
| MC.09.04.b034 | Drag the selected-portion borders with a mouse or trackpad | Adjusts the selected portion to record. | A screen portion has been selected in Screenshot. |
| MC.09.04.b035 | Choose a microphone in recording options | Records your voice or other audio with the screen recording. | Screenshot or QuickTime Player recording. |
| MC.09.04.b036 | Choose Include System Audio | Records system audio with the screen recording. | Screenshot app; macOS 27 Golden Gate or later. |
| MC.09.04.b037 | Choose Show Mouse Clicks | Shows a black circle around the pointer when you click. | Screenshot recording options. |
| MC.09.04.b038 | Choose Show Mouse Clicks in Recording | Shows a black circle around the pointer when you click. | QuickTime Player recording settings. |
| MC.09.04.b039 | Choose a number of seconds for the recording timer | Delays the start of recording by that number of seconds after you click Record. | Screenshot recording options. |
| MC.09.04.b040 | Choose a different Save to location | Changes where the recording is saved after you stop recording. | Screenshot app; default save location is the desktop. |
| MC.09.04.b041 | Choose SDR (Most Compatible) | Captures standard dynamic range using H.264. | Supported Mac models using macOS Tahoe 26 or later. |
| MC.09.04.b042 | Choose HDR | Captures high dynamic range using HEVC. | Supported Mac models using macOS Tahoe 26 or later. |
| MC.09.04.b043 | Press Esc before clicking Record | Cancels making a recording. | Screenshot app. |
| MC.09.04.b044 | Click Record in the Screenshot toolbar | Starts recording. | Screenshot app; for a selected-window recording, click the window to record. |
| MC.09.04.b045 | Click the window to record | Starts recording that window. | Record Selected Window mode. |
| MC.09.04.b046 | Click Stop in the menu bar or press Command-Control-Esc | Stops recording. | Screenshot or QuickTime Player recording. |
| MC.09.04.b047 | Click the recording thumbnail | Opens the recording for editing. | Thumbnail appears in the corner of the screen after recording. |
| MC.09.04.b048 | Wait after the recording thumbnail appears | The recording saves to the desktop. | Screenshot app. |
| MC.09.04.b049 | Open QuickTime Player from the Applications folder | Opens QuickTime Player. | To use QuickTime Player for a screen recording. |
| MC.09.04.b050 | Choose New Screen Recording from the File menu | Opens screen-recording controls. | QuickTime Player. |
| MC.09.04.b051 | Press Control-Command-N | Opens screen-recording controls. | QuickTime Player. |
| MC.09.04.b052 | See the onscreen controls described for Screenshot | Follow the Screenshot app recording steps. | After starting New Screen Recording in QuickTime Player; indicates screen recording is performed by Screenshot. |
| MC.09.04.b053 | See the Screen Recording window | Continue with the QuickTime Player recording steps. | After starting New Screen Recording in QuickTime Player; indicates screen recording is performed by QuickTime Player. |
| MC.09.04.b054 | Choose a microphone in QuickTime Player | Records your voice or other audio with the screen recording. | QuickTime Player recording settings. |
| MC.09.04.b055 | Adjust the volume slider | Adjusts audio monitoring volume during recording. | QuickTime Player; if audio feedback occurs, lower the volume or use headphones with a microphone. |
| MC.09.04.b056 | Press Esc | Cancels making a recording. | QuickTime Player, before recording starts. |
| MC.09.04.b057 | Click Record, then click anywhere on the screen | Begins recording the entire screen. | QuickTime Player. |
| MC.09.04.b058 | Click Record, drag to select an area, then click Start Recording within that area | Begins recording the selected area. | QuickTime Player. |
| MC.09.04.b059 | Stop a QuickTime Player recording | QuickTime Player automatically opens the recording. | After recording stops; the recording can then be played, edited, or shared. |
| MC.09.04.b060 | Try to record a window in some apps | The app might not let you record its window. | Some apps, such as the Apple TV app. |
| MC.09.04.b061 | Open a screen recording | The recording can be viewed or edited. | QuickTime Player, iMovie, and other apps that can edit or view videos. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.09.04.k008 | Shift-Command-5 | Open the Screenshot app. | Screen recording on Mac. |
| MC.09.04.k009 | Esc (Escape) | Cancel making a recording. | Before starting a recording; Screenshot says before clicking Record, and QuickTime Player says before recording. |
| MC.09.04.k010 | Command-Control-Esc (Escape) | Stop recording. | Screenshot or QuickTime Player. |
| MC.09.04.k011 | Control-Command-N | Choose New Screen Recording in QuickTime Player. | QuickTime Player. |

macOS 27 statements on this page:

- MC.09.04.n01: In macOS 27 Golden Gate or later, Screenshot recording options include Include System Audio to record system audio.

#### Independent reports (MC.09.04)

| ID | Kind | Area | Claim | Source |
|---|---|---|---|---|
| MC.09.04.x01 | new | screenshots | Screen recording supports system audio. | T3 [Wikipedia](https://en.wikipedia.org/wiki/MacOS_Golden_Gate) |
| MC.09.04.x02 | behavior | screenshots | On macOS 27.0, setting only `location-screenshot` saves screenshots taken with Shift-Command-3 to the designated folder. | T5 [Reddit r/MacOS](https://www.reddit.com/r/MacOS/comments/1xd2vkn/psa_on_macos_27_defaults_write/) |
| MC.09.04.x03 | changed | screenshots | On macOS 27.0, setting only the older `location` key leaves screenshots saving to the Desktop. | T5 [Reddit r/MacOS](https://www.reddit.com/r/MacOS/comments/1xd2vkn/psa_on_macos_27_defaults_write/) |
| MC.09.04.x04 | behavior | screenshots | The `location-last` key stores the folder shown in the Shift-Command-5 Options menu; it does not represent the current screenshot save destination. | T5 [Reddit r/MacOS](https://www.reddit.com/r/MacOS/comments/1xd2vkn/psa_on_macos_27_defaults_write/) |
| MC.09.04.x05 | workaround | screenshots | To set a screenshot destination on macOS 27, the post recommends creating the folder, setting `location-screenshot`, and restarting SystemUIServer. | T5 [Reddit r/MacOS](https://www.reddit.com/r/MacOS/comments/1xd2vkn/psa_on_macos_27_defaults_write/) |
| MC.09.04.x06 | workaround | screenshots | To revert to saving screenshots on the Desktop, the post recommends deleting `location-screenshot` and restarting SystemUIServer. | T5 [Reddit r/MacOS](https://www.reddit.com/r/MacOS/comments/1xd2vkn/psa_on_macos_27_defaults_write/) |
| MC.09.04.x07 | workaround | screenshots | For scripts used across macOS versions, the post recommends setting both `location` and `location-screenshot`; older versions recognize `location` and disregard the new key. | T5 [Reddit r/MacOS](https://www.reddit.com/r/MacOS/comments/1xd2vkn/psa_on_macos_27_defaults_write/) |
| MC.09.04.x08 | workaround | screenshots | The post recommends Shift-Command-5, then Options, as the straightforward way to change the screenshot location without a script. | T5 [Reddit r/MacOS](https://www.reddit.com/r/MacOS/comments/1xd2vkn/psa_on_macos_27_defaults_write/) |
| MC.09.04.x09 | behavior | screenshots | The screenshot shortcuts and Screenshot app options are unchanged from macOS 26 Tahoe; screenshots still save to the Desktop by default. | T4 [Pizazoo](https://pizazoo.com/blog/macos-27-screenshot-features/) |
| MC.09.04.x10 | behavior | screenshots | The Screenshot app Options menu includes controls for save location, timer, floating thumbnail, remembered selection, mouse pointer, and capture format. | T4 [Pizazoo](https://pizazoo.com/blog/macos-27-screenshot-features/) |
| MC.09.04.x11 | behavior | screenshots | On supported Macs, SDR (Most Compatible) capture format saves PNG files, while HDR saves HEIF files. | T4 [Pizazoo](https://pizazoo.com/blog/macos-27-screenshot-features/) |
| MC.09.04.x12 | new | screenshots | A third-party report describes a Visual Intelligence button in the Shift-Command-5 toolbar; the article says Apple's guide does not list it. | T4 [Pizazoo](https://pizazoo.com/blog/macos-27-screenshot-features/) |
| MC.09.04.x13 | bug | screenshots | Fixed an issue where HDR screenshots could appear garbled when sent in Messages. | T4 [BetaProfile](https://betaprofile.dev/macos-27-rc/) |

## 14.10 MC.10 Session


### MC.10.01 Login window

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).

Also covered by: [Log out of your account on your Mac - Apple Support](https://support.apple.com/guide/mac-help/mchlcc122483/27/mac/27)


#### Turn on accessibility options in the login window on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlaa57f797/27/mac/27) · macOS 27 text present: no

The article explains how to enable selected accessibility options in the Mac login window and describes their availability for all users.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.10.01.e001 | Apple menu | menu | At the start of the instructions | The menu the article says to choose. |
| MC.10.01.e002 | Open Lock Screen settings for me | other | After step 1 | Link to Lock Screen settings. |
| MC.10.01.e003 | VoiceOver | setting | Step 2 | An example of an accessibility option that can be enabled in the login window. |
| MC.10.01.e004 | Zoom | setting | Step 2 | An example of an accessibility option that can be enabled in the login window. |
| MC.10.01.e005 | Accessibility Keyboard | setting | Step 2 | An accessibility option; the article notes that its current settings apply in the login window for all users if selected. |
| MC.10.01.e006 | Switch Control | setting | Step 2 and See also | An accessibility option; the article notes that current switches settings apply in the login window for all users if selected. |
| MC.10.01.e007 | Done | button | Step 3 | The button the article says to click after selecting options. |
| MC.10.01.e008 | Accessibility shortcuts | other | After step 3 | Shortcuts that people logging in can use to temporarily turn accessibility options on or off in the login window. |
| MC.10.01.e009 | Accessibility settings | setting | After step 3 | The settings where an accessibility option can be turned on to continue using it after logging in. |
| MC.10.01.e010 | Open Accessibility settings for me | other | After the instruction to choose the Apple menu | Link to Accessibility settings. |
| MC.10.01.e011 | Change Accessibility settings on Mac | other | See also | Related article link. |
| MC.10.01.e012 | Use Voice Control commands to interact with your Mac | other | See also | Related article link. |
| MC.10.01.e013 | Use Switch Control on Mac | other | See also | Related article link. |
| MC.10.01.e014 | Use the Accessibility Keyboard on Mac | other | See also | Related article link. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.10.01.b001 | Turn on an accessibility option in Lock Screen settings for the login window. | The selected option is enabled in the login window for all users of the Mac. | The option must be selected; an administrator name and password may be required. |
| MC.10.01.b002 | Select Accessibility Keyboard or Switch Control. | The current keyboard or switches settings are applied in the login window for all users. | Applies when either named option is selected. |
| MC.10.01.b003 | Click Done. | The article's setup steps are completed. | After selecting the desired accessibility options. |
| MC.10.01.b004 | A person logging in uses Accessibility shortcuts. | Accessibility options can be temporarily turned on or off in the login window. | Available to anyone who logs in to the Mac; the article does not specify shortcut keys. |
| MC.10.01.b005 | Turn an accessibility option on in Accessibility settings. | The option can continue to be used after logging in to the Mac. | Applies when the option was used in the login window and the user wants to keep using it after login. |

System Settings paths referenced: Lock Screen settings; Accessibility settings

#### Independent reports (MC.10.01)

| ID | Kind | Area | Claim | Source |
|---|---|---|---|---|
| MC.10.01.x01 | new | login | In the macOS 27 Golden Gate beta, under “Ustawienia systemowe” (System Settings) > “Ogólne” (General) > “Logowanie i rozszerzenia” (Login Items & Extensions), you can see in real time which listed apps are performing background operations; you can also grant or revoke permission to run processes in the background. (translated) | T4 [iMagazine](https://imagazine.pl/2026/06/10/macos-27-golden-gate-aplikacje-dzialajace-w-tle-dock/) |
| MC.10.01.x02 | new | login | According to the article, there is a section for apps’ background activity under “Allgemein” (General) > “Anmeldeobjekte” (Login Items). macOS shows which apps are running such processes there; permissions for background activity can be enabled or disabled. (translated) | T4 [iFun](https://www.ifun.de/macos-27-dock-zeigt-aktive-hintergrund-apps-an-285370/) |
| MC.10.01.x03 | behavior | login | Accessibility features enabled in Accessibility settings also need permission to work at the login window. | T4 [AbilityNet](https://api.abilitynet.org.uk/how-to-enable-the-accessibility-options-on-the-login-window-in-macos-27-golden-gate) |
| MC.10.01.x04 | behavior | login | VoiceOver, Zoom, Accessibility Keyboard, Sticky Keys, Slow Keys, Mouse Keys, and Switch Control can be enabled for use at the login window using toggle switches. | T4 [AbilityNet](https://api.abilitynet.org.uk/how-to-enable-the-accessibility-options-on-the-login-window-in-macos-27-golden-gate) |
| MC.10.01.x05 | setup | login | Turn on the desired accessibility-option toggles and click Done to save the changes. | T4 [AbilityNet](https://api.abilitynet.org.uk/how-to-enable-the-accessibility-options-on-the-login-window-in-macos-27-golden-gate) |
| MC.10.01.x06 | behavior | login | Only someone with administrator-level login access can enable this feature. | T4 [AbilityNet](https://api.abilitynet.org.uk/how-to-enable-the-accessibility-options-on-the-login-window-in-macos-27-golden-gate) |
| MC.10.01.x07 | new | login | With `UserCreation.NewUserAuthenticationMethods` set to `OpenID`, macOS displays a web view with the identity provider’s sign-in form at the login window. The article says it supports multi-step and multi-factor authentication, including QR-code sign-in. | T4 [Stabilise](https://stabilise.io/blog/platform-sso-macos-27-mac-login-without-microsoft) |
| MC.10.01.x08 | new | login | The web-based identity-provider sign-in view is available for FileVault unlock. | T4 [Stabilise](https://stabilise.io/blog/platform-sso-macos-27-mac-login-without-microsoft) |
| MC.10.01.x09 | behavior | login | In password-sync mode, the local Mac password and identity-provider password are kept in sync; changing one causes the other to follow. | T4 [Stabilise](https://stabilise.io/blog/platform-sso-macos-27-mac-login-without-microsoft) |
| MC.10.01.x10 | behavior | login | In Secure Enclave key mode, the Mac uses a hardware-bound key for authentication, so after setup the user does not need to type a synced password for authentication. | T4 [Stabilise](https://stabilise.io/blog/platform-sso-macos-27-mac-login-without-microsoft) |
| MC.10.01.x11 | behavior | login | The `Policies.OfflineGracePeriod` setting defines how many days a user can continue logging in locally before the Mac requires a fresh identity-provider check. | T4 [Stabilise](https://stabilise.io/blog/platform-sso-macos-27-mac-login-without-microsoft) |
| MC.10.01.x12 | new | login | The article says `RequireTouchID` and `RequireTouchIDOrWatch` let administrators require biometric authentication by policy. | T4 [Stabilise](https://stabilise.io/blog/platform-sso-macos-27-mac-login-without-microsoft) |
| MC.10.01.x13 | changed | login | Authenticated Guest Mode is extended to Macs protected by FileVault. | T4 [Stabilise](https://stabilise.io/blog/platform-sso-macos-27-mac-login-without-microsoft) |
| MC.10.01.x14 | new | login | The new Golden Gate Bridge wallpapers animate when the Mac unlocks and can also be used as screen savers. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MC.10.01.x15 | behavior | login | The login window shows registered accounts with their names and login pictures, a password field, a background, and helper controls such as the clock, input-source switching, and accessibility options. | T4 [Koffret](https://koffret.com/learn/mac-login-screen) |
| MC.10.01.x16 | behavior | login | The login-window background is not offered as a user setting for replacing it with a personal photo. | T4 [Koffret](https://koffret.com/learn/mac-login-screen) |
| MC.10.01.x17 | behavior | login | On macOS Ventura 13 and later, automatic login is set in Apple menu > System Settings > Users & Groups by choosing an account from “Automatically log in as” and entering its password. | T4 [Koffret](https://koffret.com/learn/mac-login-screen) |
| MC.10.01.x18 | behavior | login | On earlier macOS versions, automatic login is set in System Preferences > Users & Groups after unlocking the pane, choosing Login Options, and selecting an account from “Automatic login.” | T4 [Koffret](https://koffret.com/learn/mac-login-screen) |
| MC.10.01.x19 | behavior | login | Automatic login removes the password prompt during startup, but a login may still be required after manually logging out, switching back from another user account, or locking the screen. | T4 [Koffret](https://koffret.com/learn/mac-login-screen) |
| MC.10.01.x20 | behavior | login | The automatic-login control may be unavailable when FileVault is on, an organization’s supervision profile prohibits automatic login, or the account uses an Apple Account password to log in. | T4 [Koffret](https://koffret.com/learn/mac-login-screen) |
| MC.10.01.x21 | behavior | login | A login picture can be changed in System Settings > Users & Groups by clicking the picture next to the login name. Available choices include Memoji, emoji, a monogram, a camera photo, a photo-library image, and suggestions. | T4 [Koffret](https://koffret.com/learn/mac-login-screen) |
| MC.10.01.x22 | behavior | login | When editing a login picture, dragging within the circle repositions it, a slider changes its zoom, and Save commits the change; an image file can also be dragged from Finder onto the current picture. | T4 [Koffret](https://koffret.com/learn/mac-login-screen) |
| MC.10.01.x23 | behavior | login | The picture of another user who is currently logged in cannot be changed; that user is indicated by a checkmark on their picture. | T4 [Koffret](https://koffret.com/learn/mac-login-screen) |
| MC.10.01.x24 | behavior | login | Login-window accessibility options are reached through System Settings > Lock Screen > Accessibility Options. Some may require an administrator name and password. | T4 [Koffret](https://koffret.com/learn/mac-login-screen) |
| MC.10.01.x25 | behavior | login | Accessibility options enabled for the login window apply to all users of the Mac. If Accessibility Keyboard or Switch Control is selected, the current keyboard or switch settings are applied there for all users. | T4 [Koffret](https://koffret.com/learn/mac-login-screen) |
| MC.10.01.x26 | behavior | login | Anyone logging in can use accessibility shortcuts to turn accessibility options on or off temporarily at the login window. | T4 [Koffret](https://koffret.com/learn/mac-login-screen) |
| MC.10.01.x27 | behavior | login | Login-window accessibility settings are separate from the logged-in session’s settings; for example, enabling Zoom only at the login window does not keep magnification after the desktop appears. | T4 [Koffret](https://koffret.com/learn/mac-login-screen) |

### MC.10.02 Fast user switching

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).


#### Switch quickly between users on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp2439/27/mac/27) · macOS 27 text present: no

Fast user switching lets users switch between accounts when multiple users are logged in. It can be shown in the menu bar or Control Centre, and accounts can also be switched using Touch ID.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.10.02.e001 | Apple menu | menu | Menu bar | Menu identified as the starting point for opening Menu Bar settings. |
| MC.10.02.e002 | Menu Bar settings | view | Opened from the Apple menu | Settings area where fast user switching can be configured. |
| MC.10.02.e003 | Fast User Switching | setting | Menu Bar settings and Add Controls | Control for adding fast user switching to the menu bar or Control Centre. |
| MC.10.02.e004 | pop-up menu | control | To the right of Fast User Switching in Menu Bar settings | Offers an option for how fast user switching is shown. |
| MC.10.02.e005 | Add Controls | button | Menu Bar settings | Opens controls that can be added to the menu bar or Control Centre. |
| MC.10.02.e006 | Add to Menu Bar | menu item | Fast User Switching controls | Adds fast user switching to the menu bar. |
| MC.10.02.e007 | Add to Control Centre | menu item | Fast User Switching controls | Adds fast user switching to Control Centre. |
| MC.10.02.e008 | Done | button | Menu Bar settings | Finishes the configuration steps. |
| MC.10.02.e009 | fast user switching menu | menu | Upper-right corner of the screen, in the menu bar | May use a user icon or show the current user's name, depending on Menu Bar settings. |
| MC.10.02.e010 | user name or icon | menu item | Fast user switching menu or Control Centre | Select a user to switch accounts. |
| MC.10.02.e011 | password | control | After choosing a user from the menu bar | Enter the password for the selected account. |
| MC.10.02.e012 | Return | other | Keyboard | Key pressed after entering the password. |
| MC.10.02.e013 | Touch ID | control | Mac or Magic Keyboard | Can be used to switch users when two or more users are logged in at the same time. |
| MC.10.02.e014 | Touch Bar | view | Mac with a Touch Bar | Displays the message “Press Touch ID to switch to [your name]” during the Touch ID switching procedure. |
| MC.10.02.e015 | Magic Keyboard | other | Connected keyboard | A Magic Keyboard with Touch ID only (no Touch Bar) supports the described Touch ID switching procedure. |
| MC.10.02.e016 | Control Centre | panel | Menu bar | Can contain the fast user switching control. |
| MC.10.02.e017 | fast user switching icon | icon | Control Centre | The article says to use the icon, but the icon itself is not shown in the supplied text. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.10.02.b001 | An administrator turns on fast user switching | Users can quickly switch between accounts. | The Mac has multiple users, and more than one user is logged in at the same time. |
| MC.10.02.b002 | Choose Apple menu, then open Menu Bar settings; select Fast User Switching and choose an option from the pop-up menu on the right | Fast user switching is configured to appear in the menu bar. | — |
| MC.10.02.b003 | Choose Apple menu, open Menu Bar settings, click Add Controls, click Fast User Switching on the right, choose Add to Menu Bar, then click Done | Fast user switching is added to the menu bar. | — |
| MC.10.02.b004 | Choose Apple menu, open Menu Bar settings, click Add Controls, click Fast User Switching on the right, choose Add to Control Centre, then click Done | Fast user switching is added to Control Centre. | — |
| MC.10.02.b005 | Click the fast user switching menu in the menu bar and choose a user name or icon | The selected account is chosen for switching. | The menu is in the upper-right corner of the screen. Depending on Menu Bar settings, it uses a user icon or shows the current user's name. |
| MC.10.02.b006 | Enter the password and press Return | The selected account is accessed. | After choosing a user from the menu bar. |
| MC.10.02.b007 | Place a finger on Touch ID until “Press Touch ID to switch to [your name]” appears in the Touch Bar, then press Touch ID until it clicks | Switches to the named user. | Two or more users are logged in at the same time, and the Mac has a Touch Bar. |
| MC.10.02.b008 | Press Touch ID until it clicks | Switches users. | Two or more users are logged in at the same time, and the Mac or Magic Keyboard has Touch ID only, with no Touch Bar. |
| MC.10.02.b009 | Click the fast user switching control in Control Centre, then choose a username or icon | Switches to the chosen account. | Fast user switching is available in Control Centre. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.10.02.k001 | Return | Submit the entered password. | After choosing a user from the menu bar. |
| MC.10.02.k002 | Touch ID — press and hold until it clicks | Switch users. | Two or more users are logged in at the same time; on a Mac with a Touch Bar, first wait for the switch prompt and then press Touch ID until it clicks. |

System Settings paths referenced: Apple menu > Menu Bar settings

### MC.10.03 Lock

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).


#### Lock the screen of your Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchl8e8b6a34/27/mac/27) · macOS 27 text present: no

The page explains ways to lock and unlock a Mac, and how to change what appears on the Lock Screen.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.10.03.e001 | Apple menu | menu | On the Mac | The page says to choose this menu to lock the screen and to open Lock Screen settings, but does not specify a menu item. |
| MC.10.03.e002 | Hot Corners | setting | Mac | Can be used to lock the screen. |
| MC.10.03.e003 | Touch ID | button | Mac or Magic Keyboard | Can be used to lock or unlock the Mac when available. |
| MC.10.03.e004 | Lock Screen button | button | Magic Keyboard | Can be used to lock the screen if available. |
| MC.10.03.e005 | Lock Screen | window | Mac | The screen shown when the Mac is locked. It can show the Screen Saver after a period of inactivity. |
| MC.10.03.e006 | Apple Watch | other | Used with the Mac | Can be used to unlock the Mac. |
| MC.10.03.e007 | Open Lock Screen settings for me | other | Link under the Lock Screen settings instructions | Opens Lock Screen settings. |
| MC.10.03.e008 | Show password hints | setting | Lock Screen settings | Turn on this setting to show password hints. |
| MC.10.03.e009 | Show message when locked | setting | Lock Screen settings | Turn on this setting to display a message in the login window. |
| MC.10.03.e010 | Set | button | Beside “Show message when locked” in Lock Screen settings | Opens the step for entering the message to display. |
| MC.10.03.e011 | OK | button | After entering a message | Confirms the message. |
| MC.10.03.e012 | login window | window | After the message is configured | Displays the message entered for the locked Mac. |
| MC.10.03.e013 | Screen Saver | view | Lock Screen | Appears on the Lock Screen after a period of inactivity. |
| MC.10.03.e014 | Change Lock Screen settings | other | Referenced in the page | A referenced topic about changing the period of inactivity before the Screen Saver appears. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.10.03.b001 | Choose the Apple menu | Lock the screen. | The page does not specify the menu item to choose. |
| MC.10.03.b002 | Use Hot Corners | Lock the screen. | No further conditions are stated. |
| MC.10.03.b003 | Press Touch ID | Lock the screen. | Touch ID must be available on the Mac or Magic Keyboard. |
| MC.10.03.b004 | Press the Lock Screen button | Lock the screen. | The button must be available on the Magic Keyboard. |
| MC.10.03.b005 | Press any key, then enter the password | Unlock the Mac. | The Mac is locked. |
| MC.10.03.b006 | Use Touch ID | Unlock the Mac. | Touch ID must be available on the Mac or Magic Keyboard. |
| MC.10.03.b007 | Use Apple Watch | Unlock the Mac. | No further conditions are stated. |
| MC.10.03.b008 | Move the pointer over another logged-in user's name on the Lock Screen, then click your account | Log in to your account. | Other users have accounts on the same Mac, and someone else is currently logged in. |
| MC.10.03.b009 | Turn on “Show password hints” | Show password hints. | In Lock Screen settings. |
| MC.10.03.b010 | Turn on “Show message when locked,” click Set, enter a message, then click OK | Display the entered message in the login window. | In Lock Screen settings. |
| MC.10.03.b011 | A period of inactivity elapses | The Screen Saver appears on the Lock Screen. | The inactivity period can be set; the page refers to Change Lock Screen settings. |

System Settings paths referenced: Lock Screen settings; Change Lock Screen settings

#### Independent reports (MC.10.03)

| ID | Kind | Area | Claim | Source |
|---|---|---|---|---|
| MC.10.03.x01 | setup | lock | To configure login-window accessibility options, open System Settings, select Lock Screen in the sidebar, and click Accessibility Options. | T4 [AbilityNet](https://api.abilitynet.org.uk/how-to-enable-the-accessibility-options-on-the-login-window-in-macos-27-golden-gate) |
| MC.10.03.x02 | new | lock | The web-based identity-provider sign-in view is available at the Lock Screen. | T4 [Stabilise](https://stabilise.io/blog/platform-sso-macos-27-mac-login-without-microsoft) |
| MC.10.03.x03 | behavior | lock | On macOS Ventura 13 and later, the password requirement after sleep or the screen saver is configured in Apple menu > System Settings > Lock Screen using “Require password after screen saver begins or display is turned off.” | T4 [Koffret](https://koffret.com/learn/mac-login-screen) |
| MC.10.03.x04 | behavior | lock | On earlier macOS versions, the corresponding setting is in System Preferences > Security & Privacy and is labeled “Require password after sleep or screen saver begins.” | T4 [Koffret](https://koffret.com/learn/mac-login-screen) |
| MC.10.03.x05 | behavior | lock | The password-after-sleep setting cannot be changed if iPhone Mirroring is configured to authenticate automatically. | T4 [Koffret](https://koffret.com/learn/mac-login-screen) |

### MC.10.04 Force Quit

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).

Also covered by: [Quit apps on Mac - Apple Support](https://support.apple.com/guide/mac-help/mchl834d18c2/27/mac/27)


#### How to force an app to quit on Mac

T1 · OS documentation · [Apple Support](https://support.apple.com/en-us/102586) · macOS 27 text present: no

The page explains how to quit an app normally, force an unresponsive app to quit, relaunch Finder, and restart or turn off a Mac if force quitting does not work.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.10.04.e001 | Quit | menu item | The app's menu in the menu bar | Choosing this quits the app normally. |
| MC.10.04.e002 | Safari | menu | The menu bar when Safari is the app | The example app menu containing the Quit command. |
| MC.10.04.e003 | Apple menu  | menu | The corner of the screen | Contains the Force Quit and Restart commands mentioned on the page. |
| MC.10.04.e004 | Force Quit | menu item | Apple menu  | Opens the Force Quit window. |
| MC.10.04.e005 | Force Quit window | window | Opens after choosing Force Quit or using the specified keyboard shortcut | Lets you select an unresponsive app and click Force Quit. |
| MC.10.04.e006 | Force Quit | button | Force Quit window | Forces the selected app to quit. |
| MC.10.04.e007 | Finder | other | Force Quit window | Can be selected and relaunched if it stops responding. |
| MC.10.04.e008 | Relaunch | button | Force Quit window, when Finder is selected | Quits and reopens Finder. |
| MC.10.04.e009 | Restart | menu item | Apple menu  | Restarts the Mac. |
| MC.10.04.e010 | Power button | button | On the Mac | Press and hold to turn off the Mac; press and release to turn it back on. |
| MC.10.04.e011 | Touch ID button | button | On laptop computers that have Touch ID | The button to press and hold to turn off the Mac. |
| MC.10.04.e012 | Esc | other | Left side of the Touch Bar, if the Mac has a Touch Bar | The page identifies this as the location of Esc on Macs with a Touch Bar. |
| MC.10.04.e013 | How to close an app on iPhone | other | In the section about quitting an app normally | A link to information about closing an app on iPhone. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.10.04.b001 | Press Command-Q, or choose Quit from the app's menu in the menu bar. | The app quits normally and can do what it needs to do before quitting, such as save changes. | This is the normal quitting method to try before force quitting. |
| MC.10.04.b002 | Press Option (or Alt), Command, and Esc together, or choose Force Quit from the Apple menu . | A Force Quit window opens. | Used when an app cannot be quit normally because it is not responding. On a Mac with a Touch Bar, Esc is on the left side of the Touch Bar. |
| MC.10.04.b003 | Select the unresponsive app in the Force Quit window, then click Force Quit. | The selected app is forced to quit. | The app is unresponsive. |
| MC.10.04.b004 | Select Finder in the Force Quit window, then click Relaunch. | Finder quits and reopens. | Finder stops responding. The page states that Finder is always open. |
| MC.10.04.b005 | Choose Restart from the Apple menu . | The Mac restarts. | Try this if you cannot force the app to quit. |
| MC.10.04.b006 | Press and hold the Mac's power button for up to 10 seconds, until the Mac turns off. | The Mac turns off. | Use if the Mac will not restart. On laptop computers that have Touch ID, press and hold the Touch ID button. |
| MC.10.04.b007 | Press and release the power button. | The Mac turns back on. | After turning the Mac off using the power button or Touch ID button. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.10.04.k001 | Command-Q | Quit the current app normally. | Try this before force quitting. |
| MC.10.04.k002 | Option (or Alt) + Command + Esc | Open the Force Quit window. | Used to force an app to quit when it is not responding. On a Mac with a Touch Bar, Esc is on the left side of the Touch Bar. |

#### If an app on your Mac stops responding, quits unexpectedly, or won’t open

T1 · OS documentation · [Apple Support](https://support.apple.com/en-us/102152) · macOS 27 text present: no

Apple Support suggests quitting and reopening the app, restarting the Mac, installing app and macOS updates, or deleting and reinstalling the app. If the app came from the App Store, you can also contact its developer or request a refund for a recent purchase.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.10.04.e014 | Quit and reopen the app | view | Section heading | Section about quitting an app and opening it again. |
| MC.10.04.e015 | Restart your Mac | view | Section heading | Section advising you to restart your Mac. |
| MC.10.04.e016 | Install software updates | view | Section heading | Section about checking for app updates and updating macOS. |
| MC.10.04.e017 | App Store | other | Install software updates and Get more help sections | Source from which an app may have been obtained; the page also directs users to its Updates section. |
| MC.10.04.e018 | Updates | other | App Store | Section to check for app updates. |
| MC.10.04.e019 | Delete and reinstall the app | view | Section heading | Section about deleting and redownloading an app. |
| MC.10.04.e020 | Get more help | view | Section heading | Section with additional options for apps obtained from the App Store. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.10.04.b008 | An app is open and does not work as expected. | Try to quit the app, then open it again. | If the app will not quit, force it to quit. |
| MC.10.04.b009 | An app does not work as expected. | Restart your Mac. | — |
| MC.10.04.b010 | The app was obtained from the App Store. | Open the App Store and check the Updates section for updates. | Applies to apps obtained from the App Store. |
| MC.10.04.b011 | The app was obtained from another source. | Get updates from the app developer’s website. | Applies to apps not obtained from the App Store. |
| MC.10.04.b012 | macOS needs an update. | Update macOS on your Mac. | — |
| MC.10.04.b013 | An app does not work as expected. | Delete the app, then redownload it. | Deleting the app might cause loss of data stored in it. If it was not obtained from the App Store, download it again from the app developer’s website. |
| MC.10.04.b014 | The app was obtained from the App Store and more help is needed. | Contact the app developer. | Applies to apps obtained from the App Store. |
| MC.10.04.b015 | The app was bought recently and the user no longer wants it. | You can request a refund. | Applies to a recent purchase. |

#### If an app freezes or quits unexpectedly on Mac

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp2579/27/mac/27) · macOS 27 text present: no

Apple suggests force quitting an unresponsive app, reopening an app that quit, restarting the Mac, checking compatibility and updates, testing another file, disconnecting recently connected peripherals, removing third-party plug-ins or enhancements, and diagnosing hardware problems.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.10.04.e021 | Apple menu | menu | Not specified | The article says to choose this menu to force quit an app or restart the Mac. |
| MC.10.04.e022 | Reopen | button | In the dialog that appears after an app quits | Click to reopen the app. |
| MC.10.04.e023 | dialog | window | Appears after an app quits | The dialog offers the Reopen option. |
| MC.10.04.e024 | Mac User Guide | other | Below the suggestions | A referenced guide. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.10.04.b016 | An app does not quit as expected or is unresponsive | Force quit the app. | You may lose unsaved changes. |
| MC.10.04.b017 | An app quits | Click Reopen in the dialog that appears to reopen the app. | — |
| MC.10.04.b018 | Restarting the Mac | Choose Apple menu. | The article does not specify a menu item. |
| MC.10.04.b019 | An app freezes or quits unexpectedly | Make sure the app is compatible with your version of macOS. | — |
| MC.10.04.b020 | A file may be causing a problem | Open another file in the app. | The article says there may be a problem with the original file. |
| MC.10.04.b021 | Software may need updating | Check for updates to your software. | — |
| MC.10.04.b022 | You just connected a peripheral device, such as a printer | Disconnect the device. | The device may not be compatible with your version of macOS. |
| MC.10.04.b023 | An app or macOS was recently updated | Uninstall app plug-ins or enhancements from other manufacturers. | A plug-in or enhancement you installed may no longer be compatible. |
| MC.10.04.b024 | Hardware you installed may be causing problems | Diagnose problems. | The article gives additional memory as an example and says hardware may be incompatible, improperly installed, or defective. |

### MC.10.05 Startup/shutdown/restart

Status: **PARTIAL**. Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18).

Also covered by: [Shut down or restart your Mac](https://support.apple.com/guide/mac-help/mchlp2522/27/mac/27)


#### Log out of your account on your Mac - Apple Support

T1 · OS documentation · [Mac User Guide](https://support.apple.com/guide/mac-help/mchlcc122483/27/mac/27) · macOS 27 text present: no

Explains that you can log out of your Mac user account, optionally prevent open app windows from reopening at the next login, and allow other users to log in on a Mac set up for multiple users.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.10.05.e001 | Apple menu | menu | Mac menu bar; referenced in the instruction | The page says to choose this menu but provides no further steps in the supplied text. |
| MC.10.05.e002 | Reopen windows when logging back in | setting | Referenced in the logout instruction | The page says to deselect this option if you do not want open app windows to reopen when you log back in. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.10.05.b001 | You choose Apple menu (as stated in the supplied instruction). | The page does not provide the rest of the instruction or specify the resulting action. | — |
| MC.10.05.b002 | You log out of your user account. | You can do so at any time. | On your Mac. |
| MC.10.05.b003 | You deselect “Reopen windows when logging back in.” | App windows that are open will not reopen when you log back in. | If you do not want open app windows to reopen. |
| MC.10.05.b004 | You log out of your account. | Other users can log in. | If your Mac is set up for multiple users. |

### MC.10.06 Setup Assistant

Status: **PARTIAL**. Only a model-specific guide (MacBook Pro) describes Setup Assistant steps (GAP-B5-09).


#### Set up your MacBook Pro

T1 · OS documentation · [MacBook Pro guide](https://support.apple.com/guide/macbook-pro/set-up-your-mac-apd831707cb3/mac) · macOS 27 text present: yes

The support page explains how to power on and connect a MacBook Pro, complete Setup Assistant, and proceed after setup. It describes setup choices for connectivity, data transfer, accessibility, accounts, and other settings.

| ID | Element (label) | Kind | Location | Description |
|---|---|---|---|---|
| MC.10.06.e001 | Setup Assistant | view | Initial setup | Guides the user through setting up the MacBook Pro. |
| MC.10.06.e002 | select a language | control | Beginning of Setup Assistant | The page says to scroll to this step to get started. |
| MC.10.06.e003 | Choose how to set up your Mac | view | Setup Assistant | Offers setup with an iPhone or iPad, data transfer, or setting up the Mac as new. |
| MC.10.06.e004 | Connect to a Wi-Fi network | view | Setup Assistant | Offers a choice of Wi-Fi network and a password field if needed. |
| MC.10.06.e005 | Other Network options | control | Network setup in Setup Assistant | An option to click when using Ethernet. |
| MC.10.06.e006 | Turn on Accessibility features | view | Setup Assistant | Offers accessibility options for vision, hearing, mobility, speech, and cognition. |
| MC.10.06.e007 | Not Now | button | Accessibility setup in Setup Assistant | Skips accessibility setup for now. |
| MC.10.06.e008 | Create your computer account and password | view | Setup Assistant | Provides fields to enter a name and create a password. |
| MC.10.06.e009 | Sign in to your Apple Account | view | Setup Assistant | Provides the option to sign in with an Apple Account or create one. |
| MC.10.06.e010 | Finish setting up your Mac | view | Setup Assistant | Includes instructions to customize settings after accounts are set up. |
| MC.10.06.e011 | Location Services | setting | Finish setting up your Mac | A setting that can be customized during final setup. |
| MC.10.06.e012 | Screen Time | setting | Finish setting up your Mac | A setting that can be customized during final setup. |
| MC.10.06.e013 | Apple Intelligence | setting | Finish setting up your Mac | A setting that can be customized during final setup. |
| MC.10.06.e014 | Siri | setting | Finish setting up your Mac | A setting that can be customized during final setup. |
| MC.10.06.e015 | Touch ID | setting | Finish setting up your Mac | A setting that can be customized during final setup. |
| MC.10.06.e016 | Welcome | view | After Setup Assistant | A message that indicates the new MacBook Pro is ready to use. |
| MC.10.06.e017 | Charge the MacBook Pro battery | indicator | Battery charging information | The page uses this label in the sentence describing how to tell that the battery is charging. |

| ID | Trigger | Result | Conditions |
|---|---|---|---|
| MC.10.06.b001 | Open the MacBook Pro lid | The Mac is ready to set up. | When starting setup. |
| MC.10.06.b002 | Begin using the new MacBook Pro | The Mac can be used immediately because its battery comes charged. | As described for a new MacBook Pro. |
| MC.10.06.b003 | Connect the MacBook Pro to a power outlet using the included power adapter and cable | The battery can be fully charged. | Recommended if the user wants to fully charge the battery. |
| MC.10.06.b004 | Connect the MacBook Pro to a power outlet using the included power adapter and cable | The Mac is powered while data is transferred. | Recommended if the user plans to transfer data to the new Mac. |
| MC.10.06.b005 | Scroll to select a language in Setup Assistant | The user can proceed to the next step. | At the beginning of Setup Assistant. |
| MC.10.06.b006 | Slide two fingers up or down on the trackpad | The page scrolls. | When scrolling on a Mac. |
| MC.10.06.b007 | Choose setup with an iPhone or iPad | The Mac is set up using that device. | As one of the Setup Assistant setup choices. |
| MC.10.06.b008 | Choose transfer from another Mac, a Time Machine backup, or a startup disk | Data is transferred to the new Mac. | As one of the Setup Assistant setup choices. |
| MC.10.06.b009 | Choose transfer from a Windows PC | Data is transferred to the new Mac. | As one of the Setup Assistant setup choices. |
| MC.10.06.b010 | Choose to set up the Mac as new | The Mac is set up without choosing one of the listed data-transfer sources. | As one of the Setup Assistant setup choices. |
| MC.10.06.b011 | Choose a Wi-Fi network and enter its password if necessary | The Mac connects to that Wi-Fi network. | During network setup. |
| MC.10.06.b012 | Click “Other Network options” | The user can follow the onscreen instructions for Ethernet. | When using Ethernet. |
| MC.10.06.b013 | Choose accessibility options | Accessibility features are set up for vision, hearing, mobility, speech, or cognition. | During Setup Assistant. |
| MC.10.06.b014 | Click “Not Now” | Accessibility setup is skipped and can be completed later. | During accessibility setup. |
| MC.10.06.b015 | Enter a name and create a password | A computer account is created; the username and password can be used to unlock the Mac or approve other actions. | During account setup. |
| MC.10.06.b016 | Add users and groups | The Mac can have multiple users. | The page says these can be added later. |
| MC.10.06.b017 | Sign in with an Apple Account email address or phone number and password | The user signs in to their Apple Account. | If the user already has an Apple Account. |
| MC.10.06.b018 | Create an Apple Account | The user can use a newly created Apple Account. | If the user does not already have an Apple Account. |
| MC.10.06.b019 | Follow the onscreen instructions after accounts are set up | The user can customize settings including Location Services, Screen Time, Apple Intelligence, Siri, and Touch ID. | During the final part of Setup Assistant. |
| MC.10.06.b020 | Complete Setup Assistant | A Welcome message informs the user that the MacBook Pro is ready to use. | After Setup Assistant is done. |
| MC.10.06.b021 | Make a Genius Bar reservation | The user can get in-person support at a nearby Apple Store. | If assistance with setup is needed. |
| MC.10.06.b022 | Contact Apple Support | The user can learn what support options are available. | If setup trouble continues. |

| ID | Keys / gesture | Action | Context |
|---|---|---|---|
| MC.10.06.k001 | Command | Open a link in a new Safari tab. | Press and hold Command while clicking one of the links described in the tip. |

macOS 27 statements on this page:

- MC.10.06.n01: The page states that Apple Intelligence on macOS 27 or later is not available in all languages or regions.
- MC.10.06.n02: The page states that Siri AI is available in beta in iOS 27, iPadOS 27, macOS 27, watchOS 27, and visionOS 27, and requires an Apple Intelligence-enabled device set to a supported language.
- MC.10.06.n03: Siri AI is stated to be available in English to start. The page says it will not initially be available in the EU on iOS, iPadOS, and watchOS.
- MC.10.06.n04: The page lists Apple Intelligence language availability and says some features may not be available in all regions or languages.
- MC.10.06.n05: The page says some Apple Intelligence features that rely on server-side models have daily usage limits, including Siri AI, Intelligent Photo Editing Tools, Image Playground, and AFM 3 Cloud models in Shortcuts. Limits may vary, and expanded access will be available for a fee in the future.

#### Independent reports (MC.10.06)

| ID | Kind | Area | Claim | Source |
|---|---|---|---|---|
| MC.10.06.x01 | behavior | setup | The article cites denied Accessibility permission as a possible cause of shortcut problems and says macOS 27 uses the permission label “Device Control and Data Access.” | T4 [Appish](https://appish.app/blog/mac-window-tiling-shortcuts-not-working-fix) |
| MC.10.06.x02 | workaround | setup | If the settings are managed by someone else, such as an IT department or administrator, contact them for access or help. | T4 [AbilityNet](https://api.abilitynet.org.uk/how-to-enable-the-accessibility-options-on-the-login-window-in-macos-27-golden-gate) |
| MC.10.06.x03 | workaround | setup | In Golden Gate Preview One, users may need to select Allow Access twice in the screen capture permissions onboarding prompt. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.10.06.x04 | fix | setup | Bartender 7 Beta Three says background permission checks no longer repeatedly ask for access and that Screen Recording permission changes are still detected correctly on macOS 27. | T4 [Bartender release notes](https://www.macbartender.com/goldengate/releases/) |
| MC.10.06.x05 | workaround | setup | The article gives this route to enable keyboard navigation: System Preferences > Keyboard > Shortcuts > “Use keyboard navigation to move focus between controls”. | T4 [tempertemper](https://www.tempertemper.net/blog/using-the-keyboard-to-navigate-on-macos) |
| MC.10.06.x06 | bug | setup | Fixed an issue where Setup Assistant could remain onscreen after configuring a new user account. | T4 [BetaProfile](https://betaprofile.dev/macos-27-rc/) |

## 14.11 Shortcut register summary (`macos_shortcuts.csv`)

| Section | Rows |
|---|---|

| Finder and system shortcuts | 80 |
| Text-editing shortcuts | 59 |
| Cut, copy, paste, and other common shortcuts | 27 |
| Keyboard focus shortcuts | 16 |
| Sleep, log out, and shut down shortcuts | 8 |
| Window tiling: Move and arrange multiple windows | 8 |
| Window tiling: Move and resize the active window | 7 |
| Vision shortcuts | 2 |
| Other accessibility shortcuts | 1 |

Notes: Apple's KB lists modifier symbols as ⌃ ⌥ ⇧ ⇪ ⌘ ⎋ ⇥ ⌫ ⏎ and arrows. On keyboards made for Windows, Alt stands in for Option, and Ctrl or the Windows key for Command ([Mac keyboard shortcuts](https://support.apple.com/en-us/102650)). Tiling shortcuts use Fn-Control combinations, for example Fn-Control-F for Fill and Fn-Control-C for Center ([Window tiling icons & keyboard shortcuts](https://support.apple.com/guide/mac-help/mchl9674d0b0/27/mac/27)).

## 14.12 Gaps and contradictions (GAP-B5-xx)

| ID | Subject | Apple evidence | Contradicting / extra evidence | Resolution path |
|---|---|---|---|---|
| GAP-B5-01 | Traffic-light redesign (supersedes Part 1 GAP-007) | No Apple description found in the release notes, guide or WWDC session 289 | Five independent sources agree from beta 6 onward ([9to5Mac](https://9to5mac.com/2026/08/17/macos-27-golden-gate-beta-6-features-redesigned-traffic-light-window-controls/), [iCulture](https://www.iculture.nl/nieuws/mac-stoplicht-knoppen-liquid-glass-macos-27/), [Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/)); light-mode pixel-quality complaints in beta ([iThinkDiff](https://www.ithinkdiff.com/macos-golden-gate-beta-6-wallpapers-traffic-light-buttons/)) | VP-18: capture active and inactive states and the press bounce on 26A434 |
| GAP-B5-02 | Dock background-process indicator and menu item | The release notes fix an incorrect background indicator ([macOS 27 release notes](https://developer.apple.com/documentation/macos-release-notes/macos-27-release-notes)), but the guide's Dock pages do not describe it ([Use the Dock](https://support.apple.com/guide/mac-help/mh35859/27/mac/27)) | Grey dot, hover label and stop item ([iFun](https://www.ifun.de/macos-27-dock-zeigt-aktive-hintergrund-apps-an-285370/)); the menu item's name is not given | Live check of the item label |
| GAP-B5-03 | Rearranging status items | Guide: hold Command and drag to rearrange or remove ([What’s in the menu bar](https://support.apple.com/guide/mac-help/mchlp1446/27/mac/27)) | "items cannot be reordered directly on the macOS 27 menu bar; macOS arranges them" ([GitHub jordanbaird/Ice](https://github.com/jordanbaird/Ice/pull/995)) | Probably a limit on programmatic reordering by third-party tools; check manually (VP-19) |
| GAP-B5-04 | Menu-bar overflow control | Not in the fetched guide pages | Double-arrow control ([MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/3/)); in beta 1 there was no setting for it ([MacGeneration](https://www.macg.co/macos/2026/06/la-barre-des-menus-de-macos-27-masque-automatiquement-ses-icones-309075)) | Search the guide again; VP-19 |
| GAP-B5-05 | Spotlight menu-bar icon | Menu Bar settings list a Spotlight control ([Menu Bar settings](https://support.apple.com/guide/mac-help/mchlad96d366/27/mac/27)) | "Spotlight was removed from the menu bar in favor of Siri for users who joined the waitlist" (beta 3; [Wikipedia](https://en.wikipedia.org/wiki/MacOS_Golden_Gate), T3) | State-dependent (Siri AI enrollment); VP-20 |
| GAP-B5-06 | Mission Control / Exposé internals and quality | Guide behaviour only ([Mission Control](https://support.apple.com/guide/mac-help/mh35798/27/mac/27)) | The Dock no longer posts the accessibility notifications that AltTab used ([GitHub lwouis/alt-tab-macos](https://github.com/lwouis/alt-tab-macos/commit/cbd8e912dded9a7f137647c1f496bfe06ed875df)); stutter reports, improved in 27.0.1 ([Reddit r/MacOS](https://www.reddit.com/r/MacOS/comments/1wlax16/mission_control_expose_is_broken_in_macos_27/), T5) | Record frame timing (VP-21) |
| GAP-B5-07 | Window corner radius and shadow values | Corner-concentricity API, no values ([WWDC26 session 289](https://developer.apple.com/videos/play/wwdc2026/289/)) | "less rounded", "more pronounced drop shadows" (qualitative) | Measure from screenshots (VP-18); Part 7 G.05 |
| GAP-B5-08 | "Ask Siri" in context menus | Spotlight "Ask Siri" documented ([Spotlight actions](https://support.apple.com/guide/mac-help/mchl4953dfeb/27/mac/27)) | At the top of every contextual menu ([Six Colors](https://sixcolors.com/post/2026/09/macos-27-golden-gate-review-bridging-the-tahoe-gap/)) | Live check; also depends on Siri AI state |
| GAP-B5-09 | Setup Assistant flow | Only a MacBook Pro guide page ([Set up your MacBook Pro](https://support.apple.com/guide/macbook-pro/set-up-your-mac-apd831707cb3/mac)) | The Liquid Glass slider appears during setup (Part 5 MD.11 reports) | Fresh-install capture (VP-22) |
| GAP-B5-10 | Screenshot save-location defaults key | Shift-Command-5 > Options documented ([Take screenshots](https://support.apple.com/guide/mac-help/mh26782/27/mac/27)) | `location-screenshot` replaces `location` on 27.0 ([Reddit r/MacOS](https://www.reddit.com/r/MacOS/comments/1xd2vkn/psa_on_macos_27_defaults_write/), T5) | Unverified; does not affect the UI model |
| GAP-B5-11 | HIG Printing page | Fetch returned 15 characters (INACCESSIBLE) | — | Retry in Part 7 |
| GAP-B5-12 | Tiling geometry | Shortcut and menu names documented ([tiling shortcuts](https://support.apple.com/guide/mac-help/mchl9674d0b0/27/mac/27)) | Settings names and conflicts with third-party tools ([Appish](https://appish.app/blog/mac-window-tiling-shortcuts-not-working-fix), a Sequoia-era article) | Edge thresholds and margins need measurement (VP-21) |
| GAP-B5-13 | Status-item hiding coupling | — | When any item is hidden, macOS also hides Focus, AirDrop, Now Playing and Fast User Switching ([GitHub thaw-app/Thaw](https://github.com/thaw-app/Thaw/blob/development/FREQUENT_ISSUES.md), T2) | Third-party observation |

## 14.13 LitricOS implications (preliminary)

| ID | Observation | Disposition |
|---|---|---|
| MC.I01 | Menu-image visibility is decided by the system, with a per-item override and SDK gating | ADOPT: a central icon-visibility policy for menus, with a per-item opt-in |
| MC.I02 | Overflow handling for status items that the notch or app menus would hide | ADOPT (concept): a first-class overflow affordance with an API, so third-party hacks aren't needed |
| MC.I03 | Third-party status-bar tools broke when `CGWindowList` exposure was removed (Ice, OnlySwitch, Thaw) | AVOID: publish a supported tray-layout API |
| MC.I04 | Background-activity visibility in the Dock, with a stop action | ADOPT: show background processes in the taskbar/dock and in Settings together |
| MC.I05 | Uniform window corner radius applied to all apps, including ones that haven't been updated | ADOPT: compositor-enforced shape tokens (Part 7/Part 10) |
| MC.I06 | Apps can suppress Hot Corners (`.disableScreenCornerInteractions`) | INVESTIGATE: an app-level opt-out for edge gestures (games, kiosks) |
| MC.I07 | Spotlight and assistant combined on one shortcut | INVESTIGATE: keep search deterministic; the assistant must be opt-in |
| MC.I08 | Copying Apple's traffic-light colours and placement verbatim | AVOID (Rule 10): design LitricOS's own window controls |

## 14.14 Validation plan additions

| ID | Test | Method | Output tag |
|---|---|---|---|
| VP-18 | Window chrome: corner radius, shadow, traffic-light active/inactive/pressed states | Screenshots at 2× on 26A434; measure in pixels | MEASURED (live) |
| VP-19 | Menu bar: Command-drag reorder, overflow control, notch behaviour | MacBook Pro 14 with notch; 25 status items | MEASURED (live) |
| VP-20 | Spotlight/Siri AI states | Enrolled vs not enrolled; record the menu-bar icon and Command–Space bar behaviour | MEASURED (live) |
| VP-21 | Mission Control, Spaces and tiling timing and thresholds | Screen recording at 120 fps; edge-drag distance | MEASURED (live) |
| VP-22 | Setup Assistant screen sequence | Fresh install on M-series desktop and laptop | MEASURED (live) |

## 14.15 Completeness audit (MC)

| ID | Leaf | Status | Basis |
|---|---|---|---|

| MC.01.01 | Apple menu | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.01.02 | App menu & standard menus | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.01.03 | Menu item image visibility (27 change) | COMPLETE | SDK-dependent menu-image rules are documented in the macOS 27 release notes (T1) and corroborated by 5 independent sources; API NSMenuItem.preferredImageVisibility. |
| MC.01.04 | Status menus / menu extras | PARTIAL | Apple documents Command-drag rearranging; macOS 27 overflow control and MenuBarAgent changes are third-party evidence only (GAP-B5-03, 04). |
| MC.01.05 | Live Activities | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.01.06 | Menu bar hiding & customization | PARTIAL | Menu Bar pane options captured (Automatically hide and show the menu bar: Always/On Desktop Only/In Full Screen Only/Never). |
| MC.02.01 | Placement/size/magnification | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.02.02 | App indicators | PARTIAL | Background-process indicator only in independent beta reports; release notes reference a fixed bug in it (GAP-B5-02). |
| MC.02.03 | Context menu incl. background processes (27) | PARTIAL | Dock shortcut menu items documented; the new background-activity menu item name is unverified (GAP-B5-02). |
| MC.02.04 | Stacks | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.02.05 | Minimize effects | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.03.01 | Windows/tabs/views | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.03.02 | Sidebar | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.03.03 | Tags | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.03.04 | Search & smart folders | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.03.05 | Quick Look | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.03.06 | Get Info | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.03.07 | Copy/move/conflict | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.03.08 | Trash & recovery | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.03.09 | Desktop & stacks | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.04.01 | App vs window relationship | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.04.02 | Active/inactive appearance | PARTIAL | Corner radius and shadow changes reported qualitatively; no numeric values published (GAP-B5-07). |
| MC.04.03 | Traffic-light controls (27 redesign) | PARTIAL | Redesign corroborated by 6 independent sources; no Apple T1 description (GAP-B5-01, supersedes Part 1 GAP-007). |
| MC.04.04 | Full screen | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.04.05 | Window tiling & Move & Resize | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.04.06 | Sheets & panels | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.04.07 | Window restoration/persistence | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.05.01 | Spaces | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.05.02 | Mission Control | PARTIAL | Guide captured; third-party reports of changed Dock accessibility notifications and animation bugs (GAP-B5-06). |
| MC.05.03 | App Exposé | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.05.04 | Stage Manager | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.06.01 | Search | PARTIAL | Spotlight guide captured; Spotlight menu-bar icon status conflicts (GAP-B5-05). |
| MC.06.02 | Actions & quick keys | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.06.03 | Intelligence/Siri integration | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.07.01 | Modules & customization | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.07.02 | Third-party controls | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.07.03 | Notification grouping | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.07.04 | Widgets on desktop | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.08.01 | Open/Save panels (Recents Cmd-Shift-F, 27) | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.08.02 | Print | PARTIAL | Mac User Guide print page captured; HIG Printing page inaccessible (GAP-B5-11). |
| MC.08.03 | Share | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.08.04 | Services menu | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.08.05 | Alerts | PARTIAL | Only HIG guidance (developer, not OS behaviour) and the NSAlert release-note item. |
| MC.09.01 | Keyboard shortcuts | COMPLETE | 208 system shortcuts parsed from Apple KB 102650 (published 2026-09-14) and the tiling-shortcut page; app-specific shortcuts are out of scope (MC.03/MD.30). |
| MC.09.02 | Trackpad gestures | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.09.03 | Full Keyboard Access | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.09.04 | Screenshots (Cmd-Shift-3/4/5) | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.10.01 | Login window | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.10.02 | Fast user switching | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.10.03 | Lock | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.10.04 | Force Quit | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.10.05 | Startup/shutdown/restart | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.10.06 | Setup Assistant | PARTIAL | Only a model-specific guide (MacBook Pro) describes Setup Assistant steps (GAP-B5-09). |

## 14.16 Source register statistics (after Part 6)

| Measure | Value |
|---|---|
| Unique URLs | 3,664 (was 2,981 after Part 5) |
| Distinct registrable domains (all statuses) | 314 (was 270) |
| Fully inspected URLs | 942 on 141 domains |
| Primary / independent (inspected) | 652 / 290 |
| Snippet-only (not counted as inspected) | 2,718 |
| Inaccessible | 4 |
| MC-linked URLs | 740 (130 inspected) |
| 400-domain target | NOT MET: 314 distinct domains in total, 141 fully inspected |
| Checklist leaves (all) | COMPLETE 18, NOT APPLICABLE 3, NOT FOUND 1, PARTIAL 313, PENDING 195 |

## 14.17 Resume point

Part 7 continues with Batch B6: the macOS design system (MG) and the macOS side of the cross-platform tokens (G): HIG, Liquid Glass, SF fonts/symbols and motion. The HIG Printing page (GAP-B5-11) will be retried.
