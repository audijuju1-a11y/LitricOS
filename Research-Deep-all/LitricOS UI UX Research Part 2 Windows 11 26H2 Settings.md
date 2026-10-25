# LitricOS UI/UX Research Encyclopedia — Part 2 of N

Chapter 11: Windows 11 26H2 Settings catalogue (Batch B1)

| Field | Value |
|---|---|
| Document ID | LITRIC-RES-P02 |
| Revision | 1.0 |
| Target release | Windows 11, version 26H2, OS build 26300.9711 (KB5127648, 2026-10-13), [Microsoft release information](https://learn.microsoft.com/en-us/windows/release-health/windows11-release-information) |
| Access date | 2026-10-25 |
| Pages catalogued | 125 Settings pages/subpages and tools |
| Controls catalogued | 1046 control rows with stable IDs (`WB.xx.yy.oNN`) |
| URI register | 237 rows / 231 distinct `ms-settings:` URIs (Appendix 2B) |
| Registry/data-store persistence rows | 382 (Appendix 2C) |
| Companion files | `litricos_coverage_checklist.csv` (rev 1.1), `litricos_source_register.csv` (rev 1.1), `ms_settings_uris.csv`, `ms_settings_registry.json` |
| Resume point | Part 3 → Batch B2 (WA: Windows shell, File Explorer, dialogs, OOBE, UAC), starting at WA.01.01 |

---

## Master Index (updated)

| Ch. | Title | Status |
|---|---|---|
| 0–6 | Baseline, checklist, plan (Part 1) | Published |
| 11 | Windows 11 26H2 Settings catalogue | PARTIAL: 129 PARTIAL, 2 COMPLETE, 1 NOT FOUND, 2 NOT APPLICABLE, 1 PENDING (this part) |
| 10 | Windows shell and system UI (WA) | Next (Part 3) |
| 12, 20–45 | Remaining chapters | PENDING |

---

## 11.0 Method, Status Rules, and Limitations

### 11.0.1 Method

1. Skeleton: the full `ms-settings:` URI table from [Microsoft Learn — Launch the Windows Settings app](https://learn.microsoft.com/en-us/windows/apps/develop/launch/launch-settings-app) was parsed (Appendix 2B).
2. Page research: each of the 125 page targets was searched on support.microsoft.com and learn.microsoft.com. The best match was fetched in full, and its controls were extracted with an instruction to keep labels verbatim and record a default only when the page states one.
3. Second pass: pages with 3 or fewer controls, or with mismatched sources, were retried against up to three alternative pages.
4. Third pass: 26H2-era features (new Start, taskbar position, Settings agent, Advanced, Shared audio, System components, Actions, Quick Machine Recovery and others) were checked against the Windows Insider blog and independent sources.
5. Persistence: registry and data-store locations were extracted from Microsoft's settings references ([common](https://learn.microsoft.com/en-us/windows/apps/develop/settings/settings-common); [Windows 11](https://learn.microsoft.com/en-us/windows/apps/develop/settings/settings-windows-11)).

### 11.0.2 Status rules applied

- COMPLETE: an official source enumerates the page fully. Only System > Advanced and the URI register qualify.
- PARTIAL: the controls are documented, but the official article is written for "Windows 11" in general rather than build 26300.9711, and full coverage of every control on the live page is not proven. Under the Part 1 rules, nothing becomes COMPLETE until it has been checked against a live 26H2 build (VP-01).
- NOT FOUND / NOT APPLICABLE: the reason is stated per item (Section 11.14).

### 11.0.3 Limitations (read before using the tables)

- L-01: Extraction was LLM-assisted. Labels marked verbatim come from the source text, but each must be re-read against the live UI before LitricOS cites it as Windows behavior.
- L-02: Microsoft support articles are evergreen and often lag behind the current build. The clearest example is GAP-B1-01 (taskbar position, Section 11.15).
- L-03: Some sources are non-English locale copies of support.microsoft.com (zh-CN, ar-SA, ja-JP, ko-KR, pl-PL), returned because they matched best. Labels taken from them were extracted in English translation and must be checked against en-US UI strings.
- L-04: Defaults are recorded only where a source states them. A "—" means the default was not documented; it does not mean there is no default.
- L-05: Controlled Feature Rollout means two devices on the same build can show different pages (GAP-005 from Part 1).
- L-06: Community Q&A (learn.microsoft.com/answers) is tier T5 and is labeled as such in the evidence column.

---

## 11.1 Settings App Shell: Structure, Navigation, Home, Search, Agent

### 11.1.1 Top-level categories (as documented)

Microsoft's two overview articles list these categories. The [accessibility walkthrough](https://support.microsoft.com/en-us/accessibility/windows/understand-and-explore-windows-settings) gives the left-navigation order as System, Bluetooth & devices, Network & internet, Personalization, Apps, Accounts, Time & language, Gaming, Accessibility, Privacy & security, Windows Update. The [Exploring Windows Settings](https://support.microsoft.com/en-us/windows/experience/exploring-windows-settings) article adds Home as the first page.

| Order | Category | Pages/topics named in the overview articles |
|---|---|---|
| 0 | Home | Recommended Settings; Personalization; Bluetooth; Storage |
| 1 | System | Display; Sound; Notifications; Storage; Power; clipboard; screensharing and projecting; Change display brightness and color in Windows; Change notification and quick settings in Windows; Using the clipboard; Screen mirroring and projecting to your PC or wireless display; Share things with nearby devices in Windows; Power and sleep; Optional features; System; Display settings in Windows |
| 2 | Bluetooth & Devices | Mobile devices; Phone Link; mouse; keyboards; speakers; headphones; Make your mouse, keyboard, and other input devices easier to use; Use a screen reader to pair a Bluetooth device in Windows; Using hearing devices with your Windows 11 PC; Use a screen reader to navigate Phone Link; Bluetooth & devices; Mouse; Keyboard; Pen; Touchpad; Hardware in Windows |
| 3 | Network & Internet | Wi-Fi; Ethernet; mobile hotspots; airplane mode; dial-Up connections; Connect to a Wi-Fi network in Windows; Setting up a wireless network in Windows; VPN; Mobile hotspot; Data usage; Network & Internet; Essential Network Settings and Tasks in Windows |
| 4 | Personalization | themes; backgrounds; colors; lock screen; taskbar settings; Text Input; Fonts; Interactive Weather on Lock Screen; Desktop Spotlight; Device Usage; Personalize your lock screen; Make Windows easier to see; Personalization based on your device usage settings and your privacy; Use a screen reader to explore and navigate Windows Desktop; Control your Dynamic Lighting devices in Windows; Change display brightness and color in Windows; Themes; Backgrounds; Colors; Lock screen; Start menu; Taskbar; Personalization; Windows personalization help |
| 5 | Apps | Offline maps; apps for websites; Startup; Uninstall or remove apps and programs in Windows; Configure startup applications in Windows; Find your way with Maps; Default apps; Startup apps; Apps |
| 6 | Accounts | Windows Hello; facial recognition; fingerprint; PIN; passwords; security keys; email and account settings; Windows Backup; Use a screen reader to back up and restore data on Windows; Configure Windows Hello; Use Windows Hello with a screen reader; Use a screen reader to navigate sign in options in Windows; Microsoft account; Work or school account; Family settings; Sign-in options; Email and app accounts; Accounts; Help for accounts in Windows |
| 7 | Time & Language | date and time; language and region; typing; speech; How to set your time and time zone; Enable text suggestions in Windows; Manage the language and keyboard/input layout settings in Windows; Use voice recognition in Windows; Date and time settings; Language preferences; Region settings; Time & Language; Set time, date, and time zone settings in Windows |
| 8 | Gaming | Game bar; Captures; Game mode; Use Game Mode while gaming on your Windows device; Customize Game Bar on Windows devices; Game streaming; Gaming; Windows gaming features and tips |
| 9 | Accessibility | Vision; text size; visual effects; mouse pointer; touch indicator; text cursor; Magnifier; color filters; contrast themes; Narrator; Hearing; hearing devices; captions; Interaction; speech; keyboard; mouse; eye control; Make it easier to focus on tasks; Make Windows easier to hear; Make Windows easier to see; Change the size of text in Windows; Set up voice access; Make your mouse, keyboard, and other input devices easier to use; Use a screen reader and keyboard to navigate the Windows Accessibility Flyout; Get started with eye control in Windows; Accessibility; Discover Windows accessibility features |
| 10 | Privacy & Security | app permissions; Windows security; antivirus; firewall; device encryption; Use a screen reader and keyboard to navigate privacy settings in Windows; General privacy settings in Windows; Microsoft Privacy Statement; Location; Camera; Microphone; App permissions; Diagnostic data; Encryption; Find a lost or stolen device; Change privacy settings in Windows |
| 11 | Windows Update | update pauses; delivery optimization; Use a screen reader to check and run Windows updates; Install Windows Updates; Windows Update Delivery Optimization and privacy; Join the Windows Insider Program and manage Insider settings; Active hours; Windows Insider Program; Windows Update |

### 11.1.2 Navigation, keyboard, and search behavior (documented)

| ID | Fact | Source |
|---|---|---|
| WB.00.F01 | Press Windows + I to open Settings. The screen reader announcement given is: “Settings window, search box; find a setting.” | [Microsoft Support](https://support.microsoft.com/en-us/accessibility/windows/understand-and-explore-windows-settings) |
| WB.00.F02 | The article directs users to press Tab until focus reaches System, then use the Up or Down arrow key to move between categories and Enter to open one. | [Microsoft Support](https://support.microsoft.com/en-us/accessibility/windows/understand-and-explore-windows-settings) |
| WB.00.F03 | Within a selected section, use Tab and arrow keys to move through options; press Backspace to return to the Settings window. | [Microsoft Support](https://support.microsoft.com/en-us/accessibility/windows/understand-and-explore-windows-settings) |
| WB.00.F04 | Press Ctrl+E to move focus to the Settings search box. Use the arrow keys to browse results and Enter to open a result. | [Microsoft Support](https://support.microsoft.com/en-us/accessibility/windows/understand-and-explore-windows-settings) |
| WB.00.F05 | The supplied article does not mention a Settings Home page, cards, an agent, or a Back button. | [Microsoft Support](https://support.microsoft.com/en-us/accessibility/windows/understand-and-explore-windows-settings) |
| WB.00.F06 | Settings can be opened by right-clicking Start and choosing Settings, using the keyboard shortcut Win+I, or using the Settings link. The source text renders the shortcut as “I.” | [Microsoft Support](https://support.microsoft.com/en-us/windows/experience/exploring-windows-settings) |
| WB.00.F07 | Settings navigation has a panel with clearly labeled categories, each represented by an icon and name. | [Microsoft Support](https://support.microsoft.com/en-us/windows/experience/exploring-windows-settings) |
| WB.00.F08 | You can find settings using the search bar with keywords such as “display” or “notifications,” or browse by category. Category pages contain grouped settings or cards and may link to subpages. | [Microsoft Support](https://support.microsoft.com/en-us/windows/experience/exploring-windows-settings) |
| WB.00.F09 | On Copilot+ PCs, improved search can interpret natural-language requests such as “Change my theme” or “View storage usage.” | [Microsoft Support](https://support.microsoft.com/en-us/windows/experience/exploring-windows-settings) |
| WB.00.F10 | The Settings agent is available on Copilot+ PCs, subject to the stated language and geography requirements. It can suggest fixes and, with the user's initiation and permission, help change settings. It only suggests settings; the user must select Apply, and can undo a change. | [Microsoft Support](https://support.microsoft.com/en-us/windows/experience/exploring-windows-settings) |
| WB.00.F11 | The agent uses the on-device Settings Mu model to match queries to settings. If it cannot confidently classify a query, standard search results appear. | [Microsoft Support](https://support.microsoft.com/en-us/windows/experience/exploring-windows-settings) |
| WB.00.F12 | The Home page is typically the first page shown and includes interactive cards such as Recommended Settings, Personalization, Bluetooth, and Storage. | [Microsoft Support](https://support.microsoft.com/en-us/windows/experience/exploring-windows-settings) |
| WB.00.F13 | The article does not mention a Back button. | [Microsoft Support](https://support.microsoft.com/en-us/windows/experience/exploring-windows-settings) |

### 11.1.3 Settings agent

The facts above document the Settings agent:
- Availability: Copilot+ PCs only, subject to language and region requirements.
- Model: it uses the on-device "Settings Mu" model.
- Behavior: it suggests settings; the user must select Apply, and the change can be undone.
- Fallback: it falls back to standard search results when it cannot classify the query.

Interaction pattern for LitricOS (classified in Part 12): a natural-language query produces a proposed change, the user confirms explicitly with Apply, and Undo is available afterwards. Without the confidence threshold, the system falls back to classic search.

### 11.1.4 Category-structure evidence from preview/independent sources


#### WB.10.00 — Privacy & security page structure

- [How to Set Up Windows 11 Privacy & Security Settings in 2026](https://helpdeskgeek.com/windows-11-privacy-security-settings-guide/): applies to Windows 11, described as “any current update”; no specific build is stated.
  - General (Privacy & security section): Not stated. Contains controls including advertising ID, locally relevant website content, suggested content in Settings, and—in some builds—Copilot or on-screen-context access.
  - Diagnostics & feedback (Privacy & security section): Not stated. Provides diagnostic-data, tailored-experience, inking and typing, and diagnostic-data deletion controls.
  - Activity history (Privacy & security section): Not stated. Provides controls to stop storing activity on the device and clear stored activity.
  - App permissions (Privacy & security section): Categories include Location, Camera, Microphone, Contacts, Calendar, and more.. Opens app-permission categories with a master access switch and, where available, per-app controls.
  - Location (App-permission category): Location services on or off; individual apps can be controlled in the list.. The master switch controls location access generally; the list below it controls individual apps.
  - Camera (App-permission category): Camera access on or off; individual apps can be controlled in the list.. The master switch controls camera access generally; the list below it controls individual apps.
  - Microphone (App-permission category): Master access on or off; individual apps can be controlled in the list.. The master switch controls microphone access generally; the list below it controls individual apps.
  - Device encryption (Privacy & security setting): On or Off.. Shows encryption status and provides a recovery-key backup action when available.
  - Recall & snapshots (Privacy & security setting): On or off is implied; no other choices stated.. Controls Recall snapshots, which the article describes as periodic screenshots used for AI search.
- [Windows 11 Privacy Settings: Optional Data, Ads and Win+H Trade ...](https://windowsforum.com/news/windows-11-privacy-settings-optional-data-ads-and-win-h-trade-offs.445709/): applies to Windows 11. The page says newer versions use the newer layout, but gives no exact Windows build. Devices without the latest Windows Update may still show the older layout. It also says the Personalized offers change was spotted in Insider builds in 2024.
  - Advertising ID (Switch under Recommendations & offers on newer builds, or General on older builds. The older label was “Let me show personalized ads by using Advertising ID.”): On or off.. Controls the Windows advertising identifier available to Windows apps that use it; turning it off does not reduce the number of ads.
  - Let Windows improve Start and search results by tracking app launches (Switch under Recommendations & offers on newer builds, or General on older builds.): On or off.. Turning it off stops Windows from using apps you open to rank Start and Search; it does not disable Windows Search or remove web results.
  - Suggested content (Listed among the controls moved from General to Recommendations & offers; no more specific UI label or behavior is given.). 
  - Tailored experiences (Setting under Diagnostics & feedback in the older naming; replaced by “Personalized offers” in newer Windows versions.): The page describes turning it off; it does not specify the full set of choices.. Uses diagnostic data to show tips, ads and recommendations. With optional diagnostic data enabled, it can also use how apps and features are used.
  - Personalized offers (Newer name for “Tailored experiences”; under Recommendations & offers.): The page describes turning it off; it does not specify the full set of choices.. Uses diagnostic data to show tips, ads and recommendations.
  - Send optional diagnostic data (Switch under Diagnostics & feedback.): On or off.. Optional data can include websites visited in Microsoft browsers, device activity and enhanced error reporting. Required diagnostic data remains on.
  - Improve inking & typing (Control under Diagnostics & feedback.). Sends samples of typing or handwriting to improve recognition and text suggestions, according to the page.
  - Store my activity history on this device (Switch on the Activity history page; the page also has a Clear history button.): On or off; history can be cleared.. Controls storing activity history on the device; the page says the Activity history page may be missing on newer builds.
  - Cloud content search (Section under Search permissions, reached from Privacy & security > Search.): Separate switches for a personal Microsoft account and a work or school account.. Controls whether cloud results from the respective account are surfaced in Windows search.
  - App permissions (Categories under Privacy & security > App permissions, including Location, Camera, Microphone and Contacts.): Access can be switched off by category and, where listed, for apps.. The page cautions that desktop apps may not appear in these lists or be controlled by these switches in the same way.
  - Online speech recognition (Switch under Privacy & security > Speech.): On or off.. Windows 11 voice typing opened with Windows key + H depends on online speech recognition; turning it off stops that voice-typing feature from working, according to the page.

#### WB.06.00 — Accounts page structure (Account settings, Windows backup)

- [Here's What Windows 11's Settings App Looks Like](https://www.howtogeek.com/738486/heres-what-windows-11s-settings-app-looks-like/): applies to Windows 11 Preview release; no build number stated.
  - Account name (Clickable account name in the upper-left corner of the Settings window; stays in the sidebar at all times.). Opens the Accounts area for changing account settings or managing accounts on the PC.

#### WB.00.00 — Settings Home page cards

- [How to use the Settings app on Windows 11](https://www.xda-developers.com/windows-11-settings/): applies to Windows 11 version 23H2 is stated as the update that added the Settings Home page. No specific build number is stated.
  - Home page cards (Cards; clicking an option navigates to its respective settings page.): Cards may show recommended settings based on settings accessed recently or frequently; Microsoft-account-related items such as OneDrive storage, recovery contacts, and Microsoft 365 subscription status; and items related to Bluetooth devices, personalization, and more.. Microsoft says the page learns from usage patterns and evolves over time.
- [What's new with the Settings app in the Windows 11 version 23H2](https://www.windowscentral.com/software-apps/windows-11/whats-new-with-the-settings-app-on-windows-11-2023-update): applies to Windows 11 version 23H2 (2023 Update); some new features were also included in the October update for version 22H2.
  - Recommended settings (Interactive Home page card). Provides recommendations for settings to consider changing based on how you use your computer.
  - Cloud storage (Interactive Home page card). Summarizes OneDrive storage and displays a warning when you are nearing capacity.
  - Account recovery (Interactive Home page card). Provides recommendations for updating Microsoft account settings with more recovery information.
  - Personalization (Interactive Home page card). Provides quick options to customize the desktop, including changing themes, enabling Windows Spotlight, and changing the color system mode.
  - Microsoft 365 (Interactive Home page card). Provides an overview of subscription benefits and options to take actions.
  - Xbox (Interactive Home page card). Shows subscription details and options to take actions, similar to the Microsoft 365 card.
  - Bluetooth Devices (Interactive Home page card). Provides quick access to manage some Bluetooth settings.

#### WB.08.00 — Gaming page structure

- [Reference for Windows 11 settings - Windows apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/apps/develop/settings/settings-windows-11): applies to Windows 11; no specific version or build is stated.
  - Gaming: Game Bar, Game Mode, Gaming Shortcuts (Section heading): Game Bar, Game Mode, Gaming Shortcuts. The section covers settings related to gaming and controls such as Game bar and gaming shortcuts.
  - Allow your controller to open Game Bar (HKCU\Software\Microsoft\GameBar\UseNexusForGameBarEnabled (REG_DWORD)): 0 or 1. Key associated with the toggle.
  - Game Mode (HKCU\Software\Microsoft\GameBar\AutoGameModeEnabled (REG_DWORD)): 0 or 1. Key associated with Gaming -> Game Mode -> Game Mode.
  - Toggles save historical video enabled. (HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR\VKMSaveHistoricalVideo (REG_DWORD)): 0 or 1. Toggles save historical video.
  - Toggles take screenshot enabled. (HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR\VKMTakeScreenshot (REG_DWORD)): 0 or 1. Toggles take screenshot.
  - Key binding for take screenshot. (HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR\VKTakeScreenshot (REG_DWORD)): ASCII value for keys. Stores the key binding for take screenshot.
  - Key binding for save historical video. (HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR\VKSaveHistoricalVideo (REG_DWORD)): ASCII value for keys. Stores the key binding for save historical video.
  - Toggles broadcast enabled. (HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR\VKMToggleBroadcast (REG_DWORD)): 0 or 1. Toggles broadcast.
  - Key binding for take broadcast. (HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR\VKToggleBroadcast (REG_DWORD)): ASCII value for keys. Stores the key binding for take broadcast.
  - Toggles camera capture enabled. (HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR\VKMToggleCameraCapture (REG_DWORD)): 0 or 1. Toggles camera capture.
  - Key binding for camera capture. (HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR\VKToggleCameraCapture (REG_DWORD)): ASCII value for keys. Stores the key binding for camera capture.
  - Toggles game bar enabled. (HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR\VKMToggleGameBar (REG_DWORD)): 0 or 1. Toggles game bar.
  - Key binding for game bar. (HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR\VKToggleGameBar (REG_DWORD)): ASCII value for keys. Stores the key binding for game bar.
  - Toggles microphone capture enabled. (HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR\VKMToggleMicrophoneCapture (REG_DWORD)): 0 or 1. Toggles microphone capture.
  - Toggles recording enabled. (HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR\VKMToggleRecording (REG_DWORD)): 0 or 1. Toggles recording.
  - Key binding for recording. (HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR\VKToggleRecording (REG_DWORD)): ASCII value for keys. Stores the key binding for recording.
  - Toggles recording indicator enabled. (HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR\VKMToggleRecordingIndicator (REG_DWORD)): 0 or 1. Toggles recording indicator.
  - Key binding for recording indicator. (HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR\VKToggleRecordingIndicator (REG_DWORD)): ASCII value for keys. Stores the key binding for recording indicator.

#### WB.11.00 — Windows Update page structure (get latest updates toggle, more options)

- [Windows message center](https://learn.microsoft.com/windows/release-health/windows-message-center): applies to Windows 11, versions 25H2 and 24H2 are mentioned as eligible starting versions for the Windows 11, version 26H2 update. No build number is stated.
  - Get the latest updates as soon as they're available (Toggle): Not stated. Having this toggle turned on is stated as a condition for the update to version 26H2 to be first available.
- [Windows Update in Windows 11/10: settings ... - Techdows](https://techdows.com/windows-update-guide.html): applies to Windows 11; no specific build is stated for the Windows Update page structure.
  - Check for updates (Button on the Windows Update page): Not stated. Checks for available updates.
  - Get the latest updates as soon as they're available (Toggle on the Windows Update page): On or off. When turned on, gets non-security fixes and new features as they roll out. Security updates arrive either way.
  - Advanced options (Sub-page): Contains settings including Active hours, Optional updates, Receive updates for other Microsoft products, and Delivery Optimization.. Provides additional Windows Update settings.
  - Active hours (Setting under Advanced options): Adjust active hours to Manually or Automatically. Lets users set active hours to keep restarts out of their working day.
  - Optional updates (Setting under Advanced options): Optional updates, including drivers that can be selected for installation. Lets users select drivers and choose Download and install.
  - Pause updates (Control on the Windows Update page): Pick a date. Pauses updates for up to 35 days; updates resume when the pause ends.
  - Update history (Page link on Windows Update; listed under More options in the uninstall instructions): Not stated. Shows update history; its Related settings include Uninstall updates.
  - Delivery Optimization (Setting under Advanced options): Allow downloads from other devices. Provides the Delivery Optimization setting.

---

## 11.2–11.13 Page-by-Page Catalogue

Each page lists its status, navigation paths, URIs, sources, and a control table. Control IDs `WB.xx.yy.oNN` are stable. Where a page has preview-era or independent evidence, it follows the official evidence in a separate list.


## WB.01 — System


### WB.01.01 — System > Display

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > System > Display`; `Start > Settings > System > Display > Identify`; `Start > Settings > System > Display > Multiple displays > Detect`; `Start > Settings > System > Display > Multiple displays > Connect`

ms-settings URIs: `ms-settings:display`, `ms-settings:screenrotation`, `ms-settings:display-advanced`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/hardware/display-graphics/how-to-use-multiple-monitors-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.01.o01 | Identify | Button |  | — | Shows a number on each display to identify it. | Available when Windows detects more than one display. |
| WB.01.01.o02 | Detect | Button |  | — | Looks for another display that is connected but not showing in Settings. | Located under Multiple displays. |
| WB.01.01.o03 | Connect | Button |  | — | Connects a wireless display that is not showing in Settings. | Located under Multiple displays. |
| WB.01.01.o04 | Arrange your displays | Drag-and-drop display arrangement |  | — | Drag displays to where you want them, then select Apply. Test the layout by moving the mouse pointer across the displays. | Available when Windows detects more than one display. |
| WB.01.01.o05 | Apply | Button |  | — | Applies the display arrangement you selected. | Used after dragging displays to arrange them. |
| WB.01.01.o06 | Multiple displays | Expandable section |  | — | Expand this section to access display options. | — |
| WB.01.01.o07 | Remember window locations based on monitor connection | Checkbox |  | — |  | Located in the Multiple displays section. |
| WB.01.01.o08 | Minimize windows when a monitor is disconnected | Checkbox |  | — | The page describes windows on an external monitor being minimized when you undock, and restored to their previous positions when you re-dock. | Located in the Multiple displays section. |
| WB.01.01.o09 | Change display brightness and color | Link |  | — |  | — |
| WB.01.01.o10 | Change display resolution, scale and layout | Link |  | — |  | — |
| WB.01.01.o11 | Change display refresh rate | Link |  | — |  | — |

- Keyboard shortcuts: Windows K opens Cast for connecting to a wireless display adapter.; Windows key + P opens the display mode choices.
- SKU / hardware / version conditions: Display arrangement options appear when Windows detects more than one display.; For a wireless display adapter, connect the adapter to an HDMI port on a newer TV, then connect the PC wirelessly.; The page describes connecting external displays to a PC, including through a dock.
- Windows 10 differences (legacy): For wireless displays, Windows 10 uses Start > Settings > System > Display > Connect to a wireless display; Windows 11 uses Multiple displays > Connect.; The Windows 10 section describes changing Display orientation under Scale and Layout, and notes that the monitor must also be physically rotated. This is not described in the Windows 11 section.; Windows 10 Detect is under Start > Settings > System > Display > Multiple displays; Windows 11 includes the additional path Multiple displays > Detect.; Windows 10 identifies displays from Start > Settings > System > Display > Rearrange your displays > Identify; the Windows 11 path is Start > Settings > System > Display > Identify.


### WB.01.01.01 — System > Display > HDR

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > System > Display > HDR`; `Settings > System > Display > HDR`; `Settings > System > Display > select an HDR-capable display > HDR`; `Windows 11: Start > Settings > System > Display > select an HDR-capable display > HDR`; `Windows 11: Settings > System > Display > select an HDR-capable display > HDR > SDR content brightness or HDR content brightness`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/hardware/display-graphics/calibrate-your-hdr-display-using-the-windows-hdr-calibration-app); [Official support article (T1)](https://support.microsoft.com/zh-CN/Windows/Hardware/Display-Graphics/hdr-settings-in-windows); [Official support article (T1)](https://support.microsoft.com/en-US/Windows/hardware/display-graphics/hdr-settings-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.01.01.o01 | HDR | Setting |  | — | Turn on HDR. | The page lists HDR being turned on as a requirement for using the Windows HDR Calibration app. |
| WB.01.01.01.o02 | Display calibration for HDR video | Calibration option |  | — | Provides calibration for HDR video. | For a built-in display that can play streaming HDR video but does not support HDR games and apps. |
| WB.01.01.01.o03 | Display capabilities | Information | HDR video streaming, games, apps and more — Supported or Not supported | — | Shows whether the selected display supports HDR video streaming, games, apps and more. | The page says to check this on the HDR settings screen. |
| WB.01.01.01.o04 | HDR video streaming | Toggle section | Expanded or collapsed; the article does not state the toggle’s exact on-screen label or choices. | — | The section can contain a battery checkbox for HDR video streaming. | The article refers to this section when describing HDR video streaming on battery. |
| WB.01.01.01.o05 | Turn off HDR when my PC is running on battery | Checkbox | Selected or deselected | The PC manufacturer might have turned HDR off by default on battery; no checkbox default is stated. | Deselecting it allows HDR to stay on when the laptop is unplugged and running on battery. | In the HDR section on an HDR-capable display; applies to running the laptop on battery. |
| WB.01.01.01.o06 | Turn off HDR video streaming when my PC is running on battery | Checkbox | Selected or deselected | — | Deselecting it allows HDR video streaming when the PC is running on battery. | In the HDR or HDR video streaming section; allowing HDR on battery reduces battery life. |
| WB.01.01.01.o07 | Use Dolby Vision mode | Checkbox | Selected or deselected | Windows automatically chooses Dolby Vision when it is available on a Dolby Vision certified monitor. | Deselect it to disable Dolby Vision and use a non-certified HDR10 mode, which may provide higher brightness in some circumstances. | Available only on Dolby Vision certified monitors. |
| WB.01.01.01.o08 | SDR content brightness | Slider | Brightness levels; exact range is not stated. | The default setting or one close to it should work well most of the time. | On an external HDR display, adjusts SDR brightness relative to HDR content. Raising it can reduce detail in darker content. Some apps may not respond until restarted. | Shown for external HDR displays. |
| WB.01.01.01.o09 | HDR content brightness | Slider | Brightness levels; exact range is not stated. | The default setting or one close to it should work well most of the time. | On a built-in HDR display, adjusts HDR brightness relative to SDR brightness. Both this setting and the display brightness setting affect HDR content. Some apps may not respond until restarted. | Shown for built-in HDR displays, such as HDR-capable laptops. |
| WB.01.01.01.o10 | HDR-capable display | Display selector | Connected displays; specific choices are not stated. | — | Select the display to configure HDR for. | Relevant when multiple displays are connected. |
| WB.01.01.01.o11 | Display requirements for HDR video | Link |  | — | Provides detailed information about hardware requirements for HDR video. | — |
| WB.01.01.01.o12 | Use HDR video streaming | Toggle | On or off; the page says the toggle may not be possible to turn on or off. | Not stated. | The page says Display capabilities can show Supported next to this item even when the HDR video streaming toggle cannot be changed. | The page attributes an unavailable toggle on HDR-capable laptops to default power settings turning HDR off on battery; plugging in or allowing HDR video streaming on battery are suggested. |
| WB.01.01.01.o13 | HDR games, apps and more | Display-capability status | Supported or Not supported; the page specifically describes Supported. | Not stated. | The page says Display capabilities may show Supported next to this item even when the HDR toggle cannot be changed. | Described in the troubleshooting text for the HDR settings screen. |

- Confirmations / errors: If an external HDR display is connected to a laptop and the HDR settings screen says Not supported next to HDR with no HDR toggle, the desktop may be duplicated across displays. The article says HDR is not supported in that configuration and recommends extending the displays.; On the HDR settings screen, capabilities may show Supported next to “Use HDR video streaming” or “HDR games, apps and more” while the HDR/HDR video streaming toggle cannot be turned on or off. The article attributes this on HDR-capable laptops to battery power; it suggests plugging in the laptop or allowing HDR video streaming on battery.; The page says HDR requires a PC and display that support HDR; the display must support HDR10 for HDR10 content.; When checking display support with DirectX Diagnostic Tool, the article identifies “AdvancedColorSupported” as indicating driver and display HDR10 support, and “AdvancedColorEnabled” as indicating HDR is currently on. It says “HDR Supported” and either “BT2020RGB” or “BT2020YCC,” plus “Eotf2084Supported,” indicate HDR10 capability; “HDR Not Supported” indicates one or more requirements are missing.
- Restart / sign-out: No sign-out instruction is stated.; The article notes that restarting a laptop on battery can result in under-saturated colors if battery power settings turn HDR off; it recommends plugging in before restarting or changing the battery setting to keep HDR on.; The article says to restart an app if changing SDR/HDR content brightness does not affect some apps, or if SDR content appears too bright and desaturated.; The page says to restart an app if changing the SDR content brightness or HDR content brightness setting does not change brightness in some apps, or if SDR content appears too bright and desaturated.
- Keyboard shortcuts: Windows logo key + R opens the Run dialog for entering dxdiag when checking HDR support.
- SKU / hardware / version conditions: HDR requires a PC and display that support HDR; HDR content and apps may also require particular hardware.; HDR requires a PC and display that support HDR; HDR10 content requires an HDR10-capable display.; SDR content brightness is described for external HDR displays; HDR content brightness is described for built-in HDR displays.; The battery-related behavior and settings are described for laptops, including laptops with HDR-capable built-in displays.; The battery-related options apply to HDR-capable laptops.; The page notes that HDR may be unavailable when an external display is connected in duplicate mode; it recommends extending the desktop to view HDR on that display.; The “Use Dolby Vision mode” checkbox is available only on Dolby Vision certified monitors.; Use Dolby Vision mode is available only on Dolby Vision certified monitors.
- Windows 10 differences (legacy): The supplied Windows 10 section does not describe the Windows 11 Use Dolby Vision mode control.; Windows 10 battery option is labeled Don’t allow HDR games and apps on battery; clear the checkbox to allow HDR games and apps on battery.; Windows 10 brightness control is labeled HDR/SDR brightness balance, rather than the Windows 11 SDR content brightness or HDR content brightness controls.; Windows 10 display selection is described under Rearrange your displays; Windows 11 says to select the HDR-capable display at the top of the screen.; Windows 10 instructions require Windows 10 version 2004 or later for the described setup procedure.; Windows 10 navigation uses Settings > System > Display > Windows HD Color settings, then the Use HDR control. It says to check that Display capabilities shows Yes next to Use HDR.


### WB.01.01.02 — System > Display > Night light

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > System > Display`; `Start > Settings > System > Display > Night light`

ms-settings URIs: `ms-settings:nightlight`

Sources: [Official support article (T1)](https://support.microsoft.com/windows/set-your-display-for-night-time-in-windows-18fe903a-e0a1-8326-4c68-fd23d7aaf136)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.01.02.o01 | Night light | Toggle | On or off | — | Turns Night light on or off; the display shows warmer colors at night. | Night light isn't available with certain drivers, such as DisplayLink or Basic Display. If the toggle is grayed out, the display driver may need updating. |
| WB.01.01.02.o02 | Strength | Slider |  | — | Adjusts the intensity of the blue light reduction. | — |
| WB.01.01.02.o03 | Schedule night light | Toggle | On or off | — | When turned on, lets you choose when Night light turns on and off. | — |
| WB.01.01.02.o04 | Sunset to sunrise | Schedule choice |  | — | Schedules Night light from sunset to sunrise. | Available after turning on Schedule night light. |
| WB.01.01.02.o05 | Set hours | Schedule choice | Custom on and off times | — | Lets you enter custom times for Night light to turn on and off. | Available after turning on Schedule night light. |
| WB.01.01.02.o06 | Open Night light settings | Link |  | — |  | — |

- Confirmations / errors: If the Night light toggle is grayed out, the display driver may need updating.; Night light isn't available if the device uses certain drivers, such as DisplayLink or Basic Display.
- Windows 10 differences (legacy): Windows 10 instructions use Settings > System > Display > Night light settings to schedule Night light. They also describe the Schedule night light toggle, Sunset to sunrise, and Set hours.


### WB.01.01.03 — System > Display > Graphics

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > System > Display`; `Start > Settings > System > Display > Graphics`

ms-settings URIs: `ms-settings:display-advancedgraphics`, `ms-settings:display-advancedgraphics-default`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/hardware/display-graphics/optimizations-for-windowed-games-in-windows-11)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.01.03.o01 | Default settings | section |  | — | Contains the switch for the system-wide setting. | — |
| WB.01.01.03.o02 | Optimizations for windowed games | switch | On or off | — | Improves performance for DirectX 10 and DirectX 11 games in windowed and borderless windowed modes by transitioning compatible games to the flip-model presentation. This can reduce frame latency and enable features such as Auto HDR and variable refresh rate on supported displays. | Turning on Auto HDR turns this on automatically. To turn it off, turn off Auto HDR first. Restart the game after changing the setting. |
| WB.01.01.03.o03 | Auto HDR | setting |  | — | Turning it on automatically turns on Optimizations for windowed games. | — |
| WB.01.01.03.o04 | Custom options for apps | section |  | — | Contains app- or game-specific graphics settings. | — |
| WB.01.01.03.o05 | Options | button |  | — | Opens the graphics settings for the selected app or game. | First select an app or game under Custom options for apps. |
| WB.01.01.03.o06 | Graphics preference | setting | Let Windows decide (Power saving); Power saving; High performance | — | Sets the graphics performance option to use for the selected app or game. | The page says these choices are available if the PC has more than one GPU. |
| WB.01.01.03.o07 | Don’t use Auto HDR | checkbox | Checked or unchecked | — | Turns off Auto HDR for the selected app or game when selected. | Available in the selected app or game's Options. |
| WB.01.01.03.o08 | Save | button |  | — | Saves the selected app or game's graphics settings. | Restart the game or app after saving. |

- Restart / sign-out: After changing app-specific graphics settings, select Save and restart the game or app.; Restart the game after turning on Optimizations for windowed games.
- SKU / hardware / version conditions: The Graphics preference choices are described for PCs with more than one GPU.; Variable refresh rate is available on supported displays.


### WB.01.01.04 — System > Display > Scale

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > System > Display`; `Start > Settings > System > Display > Scale & layout`; `Start > Settings > System > Display > Scale & layout > Scale`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/hardware/display-graphics/change-your-screen-resolution-and-layout-in-windows); [Official support article (T1)](https://support.microsoft.com/en-in/help/3025083/windows-scaling-issues-for-high-dpi-devices); [Official support article (T1)](https://support.microsoft.com/help/3025083/video-playback-settings-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.01.04.o01 | Scale | Selector (control type not specified) | Options are not listed. | — | In the Scale & layout section, select Scale, then select an option. The page says it is typically best to choose the option marked (Recommended). | If external displays are connected, first choose the display you want to change. |
| WB.01.01.04.o02 | Scale & layout | Settings section | Not stated | Not stated | In Windows 11, scroll to this section, select Scale, then select an option. The page says it is typically best to choose the option marked (Recommended). | Not stated |
| WB.01.01.04.o03 | Open your Display settings | Link | Not stated | Not stated | Link label shown in the Scale instructions. | Not stated |

- Windows 10 differences (legacy): Its setting is labelled “Change the size of text, apps, and other items”; select an option, typically the one marked (Recommended).; Windows 10 labels the scale control “Change the size of text, apps, and other items”; the page says to select an option, typically the one marked “(Recommended).”; Windows 10 navigation to Display is also Start > Settings > System > Display.; Windows 10 uses the section label “Scale and layout.”; Windows 10 uses the section label “Scale and layout” and the setting label “Change the size of text, apps, and other items”; it says to select an option, typically the one marked (Recommended).; Windows 10 uses the section label “Scale and layout” instead of “Scale & layout.”


### WB.01.02 — System > Sound

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > System > Sound`; `Start > Settings > System > Sound > Advanced > More sound settings`

ms-settings URIs: `ms-settings:sound`, `ms-settings:sound-devices`, `ms-settings:sound-defaultinputproperties`, `ms-settings:sound-defaultoutputproperties`

Sources: [Official support article (T1)](https://support.microsoft.com/instantanswers/A183AA8B-0EAC-4566-867A-98D2761A770D)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.02.o01 | Output | Selectable audio device; no specific control label stated | The audio device you want to use for playback, such as speakers or headphones | — | Selecting a device sets it as the default output device. | Choose the device you want to use for playback. |
| WB.01.02.o02 | Audio enhancements | Dropdown menu | Off | — | Set to Off to disable audio enhancements. | In Sound settings, select the audio device under Output, then find this under Advanced settings. |
| WB.01.02.o03 | More mixer settings | Settings icon/button |  | — | Opens Volume mixer settings. | Accessed next to the Volume mixer slider. |
| WB.01.02.o04 | Volume mixer settings | Volume controls and mute controls for devices | Volume levels; muted or unmuted | — | Adjust a device’s volume; muted controls are marked with an “x” and can be unmuted. | Opened using More mixer settings. |
| WB.01.02.o05 | More sound settings | Link |  | — | Opens additional sound settings, where Playback and Recording tabs and device Properties are available. | Found under Advanced. |
| WB.01.02.o06 | Playback | Tab | Output devices | — | Select a device and open its Properties. | Available in More sound settings. |
| WB.01.02.o07 | Recording | Tab | Input devices | — | Select a device and open its Properties. | Available in More sound settings. |
| WB.01.02.o08 | Properties | Device properties page/dialog |  | — | Provides access to the device’s Device usage setting. | Select a device in the Playback or Recording tab. |
| WB.01.02.o09 | Device usage | Dropdown list | Use this device (enable) | — | Check that Use this device (enable) is selected for output and input devices. | Found in device Properties. |

- Windows 10 differences (legacy): Windows 10 audio enhancements are accessed through Control Panel > Hardware and Sound > Sound > Playback > Default Device > Properties > Enhancements; the page says to select Disable all enhancements or Disable all sound effects.; Windows 10 default output devices are set in Control Panel > Hardware and Sound > Sound > Playback by selecting the device and choosing Set as Default Device.; Windows 10 device properties are accessed through Start > Settings > System > Sound. The page says to select Device properties for output and input devices and make sure the Disable checkbox is cleared.

Preview/independent evidence (26H2-era):

- [OEM support (T4)](https://support.lenovo.com/ca/en/solutions/ht515606-overview-of-windows-sound-settings-windows-11) — applies to: Windows 11; no build number stated..
  - Output (Section): Specify an output device, such as a speaker or headset, and set additional sound properties such as volume.. Used to choose the output device and configure related properties.
  - Input (Section): Specify an input device, such as a microphone, and set additional sound properties such as volume.. Used to choose the input device and configure related properties.
  - Advanced (Section): Troubleshoot common sound problems; All sound devices; Volume mixer; More sound settings.. Provides advanced sound troubleshooting and settings, including settings for sound devices and app volume.
  - Help with Sound (Option): Website links with additional sound content.. Provides links to additional sound content.
- [OEM support (T4)](https://www.asus.com/us/support/faq/1044702/) — applies to: Windows 11 and Windows 10; no build number stated..
  - Output device (Device selection in Sound settings): Internal speakers, external speakers, or headset. Choose the sound output device.
  - Input device (Device selection in Sound settings): Internal microphone, external microphone, or earphone. Choose the sound input device.
  - Volume for each app (Slider in Volume Mixer). Drag the slider to adjust volume for each app; click the app’s speaker icon to turn sound on if the app is muted.
  - Master volume (Speaker icon). Click the speaker icon to turn volume on if the computer is muted.


### WB.01.02.01 — System > Sound > Volume mixer

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > System > Sound > Volume mixer`

ms-settings URIs: `ms-settings:apps-volume`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/hardware/audio/fix-app-audio-not-working-while-system-sounds-work-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.02.01.o01 | Volume mixer | Page |  | — | Under Apps, check the affected app’s volume and output device. | Windows 11 |
| WB.01.02.01.o02 | Apps | Section |  | — | Shows app-specific audio controls. | Windows 11 |
| WB.01.02.01.o03 | volume slider | Slider |  | — | Turn up the affected app’s volume. | Windows 11; under Apps |
| WB.01.02.01.o04 | Output device | Device selection |  | — | Confirm the correct output device is selected for the affected app. | Windows 11; under Apps |

- Windows 10 differences (legacy): The Windows 10 instructions access Volume mixer by right-clicking the taskbar Volume icon and selecting Open Volume mixer. They also say to check the default device in Sounds > Playback, then return to Volume mixer to confirm the app is using the correct device.


### WB.01.03 — System > Notifications

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > System > Notifications`

ms-settings URIs: `ms-settings:notifications`

Sources: [Official support article (T1)](https://support.microsoft.com/en-US/Windows/Experience/notifications-and-do-not-disturb-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.03.o01 | Open Notifications settings | Link |  | — | Opens Notifications settings. | — |
| WB.01.03.o02 | Notifications | Toggle | On or Off | — | Turns notifications on or off. | — |
| WB.01.03.o03 | Notifications from apps and other senders | App list | Apps listed under this heading | — | Select an app to adjust its notification preferences. | — |
| WB.01.03.o04 | Turn notifications on or off | Toggle | On or Off | — | Turns notifications on or off for a specific app. | Select an app under Notifications from apps and other senders. |
| WB.01.03.o05 | Show notification banners | Toggle | Enabled or disabled | — | Controls whether an app's notifications appear as pop-up banners. | Per-app setting. |
| WB.01.03.o06 | Show notifications in notification center | Toggle | Enabled or disabled | — | Controls whether an app's notifications appear in the notification center. | Per-app setting. |
| WB.01.03.o07 | Hide content when notifications are on lock screen | Toggle | Enabled or disabled | — | Prevents sensitive notifications from showing on the lock screen. | Per-app setting. |
| WB.01.03.o08 | Play a sound | Toggle | On or off | — | Controls sound for incoming notifications. | Per-app setting. |
| WB.01.03.o09 | Priority of notifications | Priority selector | Top, High, or Normal | — | Sets the app's notification priority. High priority notifications appear above Normal ones in the notification center. | Only one app can be set as Top priority. |
| WB.01.03.o10 | Show the Windows welcome experience after updates and when signed in to show what's new and suggested | Checkbox | Checked or unchecked | — | Enables a welcome screen with new features and suggestions after updates or when you sign in. | — |
| WB.01.03.o11 | Get tips and suggestions when using Windows | Checkbox | Checked or unchecked | — | Enables helpful tips and suggestions while using Windows. | — |
| WB.01.03.o12 | Turn on do not disturb automatically | Expandable section |  | — | Expands to configure conditions for automatically turning on do not disturb. | — |
| WB.01.03.o13 | During certain time of the day | Condition |  | — | A condition for automatically turning on do not disturb. | Under Turn on do not disturb automatically. |
| WB.01.03.o14 | When duplicating the display | Condition |  | — | A condition for automatically turning on do not disturb. | Under Turn on do not disturb automatically. |
| WB.01.03.o15 | When playing a game | Condition |  | — | A condition for automatically turning on do not disturb. | Under Turn on do not disturb automatically. |
| WB.01.03.o16 | When using an app in full-screen mode | Condition |  | — | A condition for automatically turning on do not disturb. | Under Turn on do not disturb automatically. |
| WB.01.03.o17 | For the first hour after a Windows feature update | Condition |  | — | A condition for automatically turning on do not disturb. | Under Turn on do not disturb automatically. |
| WB.01.03.o18 | Calls and reminders | Checkboxes | Allow incoming calls or reminders while do not disturb is on | — | Select the checkboxes to allow incoming calls or reminders while do not disturb is on. | Under priority notification settings. |
| WB.01.03.o19 | Apps | App selection section | Apps allowed to send notifications while do not disturb is on | — | Select which apps are allowed to send notifications while do not disturb is on. | Under priority notification settings. |
| WB.01.03.o20 | Add apps | Button |  | — | Used to select apps allowed to send notifications while do not disturb is on. | Under Apps in priority notification settings. |

- Keyboard shortcuts: Windows key + N
- Windows 10 differences (legacy): For individual senders, Windows 10 uses the heading Get notifications from these senders. An app can be set On or Off; when On, its settings include Show notification banners, Show notifications in action center, Play a sound when a notification arrives, how many notifications appear, and how they are prioritized.; The Windows 10 navigation path is System > Notifications & actions.; The Windows 10 notification center shortcut stated on the page is Windows key + A.; Windows 10 has the setting Get notifications from apps and other senders, which can be set to On or Off.; Windows 10 offers the checkboxes Show notifications on the lock screen, Show reminders and incoming VoIP calls on the lock screen, and Allow notifications to play sounds.


### WB.01.03.01 — System > Notifications > Do not disturb

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > System > Notifications`

Sources: [Official support article (T1)](https://support.microsoft.com/windows/feeca47f-0baf-5680-16f0-8801db1a8466)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.03.01.o01 | Turn on do not disturb automatically | Expandable section; configure automatic activation conditions | During certain time of the day; When duplicating the display; When playing a game; When using an app in full-screen mode; For the first hour after a Windows feature update | — | Lets you configure conditions for automatically turning on Do not disturb. | — |
| WB.01.03.01.o02 | Calls and reminders | Checkboxes | Allow incoming calls; allow reminders | — | Controls whether calls and reminders are allowed while Do not disturb is on. | Under priority notification settings. |
| WB.01.03.01.o03 | Apps | Button: Add apps | Select which apps are allowed to send notifications while Do not disturb is on. | — | Lets you choose apps allowed to send notifications while Do not disturb is on. | Under priority notification settings. |
| WB.01.03.01.o04 | Add apps | Button |  | — | Opens the selection of apps allowed to send notifications while Do not disturb is on. | Under Apps. |

- Keyboard shortcuts: Windows key + N
- Windows 10 differences (legacy): The Windows 10 section describes notification settings under System > Notifications & actions and does not describe Do not disturb settings or its automatic conditions and priority notifications.; Windows 10 uses Windows key + A to open notification center.


### WB.01.04 — System > Focus

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Settings > System > Focus`

ms-settings URIs: `ms-settings:quiethours`, `ms-settings:quietmomentshome` (not in MS Learn table — unverified)

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/experience/focus-stay-on-task-without-distractions-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.04.o01 | Start focus session | Button |  | — | Starts a Focus session. | Under Focus in Settings. |
| WB.01.04.o02 | Session duration | Not stated |  | — | A setting for future focus sessions. | Under Focus in Settings. |
| WB.01.04.o03 | Show the timer in the Clock app | Not stated |  | — | A setting for future focus sessions. | Under Focus in Settings. |
| WB.01.04.o04 | Hide badges on taskbar apps | Not stated |  | — | A setting for future focus sessions. | Under Focus in Settings. |
| WB.01.04.o05 | Hide flashing on taskbar apps | Not stated |  | — | A setting for future focus sessions. | Under Focus in Settings. |
| WB.01.04.o06 | Turn on do not disturb | Not stated |  | — | A setting for future focus sessions. | Under Focus in Settings. |

- Windows 10 differences (legacy): The Windows 10 instructions use Settings > System > Focus assist. They describe Off, Priority only, and Alarms only states; Automatic rules; Show me a summary of what I missed while focus assist was on; and Show a notification in action center when focus assist is turned on automatically.

Preview/independent evidence (26H2-era):

- [Independent (T4)](https://techcommunity.microsoft.com/discussions/windows11/windows-11-how-does-the-built-in-focus-mode-timer-function/4543891) — applies to: Not stated.
  - Session duration (Plus or minus controls): Adjust the duration until it reaches 50 minutes; exact increments are not stated.. Sets the focus-session duration.
  - Start focus session (Select). Starts the focus session.
  - Do not disturb (). Windows automatically enables it during a focus session; it is configurable on the same Focus settings page.
  - Hide taskbar badges and flashing (). These options are configurable on the same Focus settings page.
- [Independent (T4)](https://www.techradar.com/how-to/how-to-use-the-focus-feature-in-windows-11-2022-update) — applies to: Windows 11 2022 Update.
  - Show the timer in the Clock app (Checkbox): On or off — default: Enabled. Shows the timer in the Clock app during a Focus session.
  - Hide badges on Taskbar apps (Checkbox): On or off — default: Enabled. Hides badges on Taskbar apps during a Focus session.
  - Hide Flashing on the taskbar app (Checkbox): On or off — default: Enabled. Hides taskbar app flashing during a Focus session.
  - Turn on Do Not Disturb (Checkbox): On or off — default: Enabled. Turns on Do Not Disturb for a Focus session.
  - Session duration (- and + buttons): 5 to 240 minutes — default: Not stated. Sets how long the Focus session should last.
  - Start Focus Session (Button): Start a session — default: Not stated. Starts the configured Focus session; changes to “Stop Focus Session” while the session is underway.
  - Notifications (Link under “Related Settings”): Opens Notifications settings — default: Not stated. Lets you configure which notifications to see when Do Not Disturb is enabled because of a Focus session. The “Set Priority Notifications” section lets you choose apps that can alert you even when Do Not Disturb is activated.
  - Requirement/rollout: The article lists Windows 11 2022 Update as the requirement.
  - Requirement/rollout: No Copilot+ PC, Insider channel, region, or SKU requirement is stated.


### WB.01.05 — System > Power & battery

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Settings > System > Power & battery`; `Settings > System > Power & battery > Power Mode`; `Settings > System > Power & battery > Energy saver`; `Settings > System > Power & battery > Screen, sleep & hibernate timeouts`; `Start > Settings > System > Power & battery > Screen, sleep, & hibernate timeouts`

ms-settings URIs: `ms-settings:powersleep`, `ms-settings:batterysaver`, `ms-settings:batterysaver-settings`, `ms-settings:batterysaver-usagedetails`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/experience/power-battery/power-settings-in-windows-11)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.05.o01 | Power Mode | Control type not stated | Best Power Efficiency is named. | — | Selecting Best Power Efficiency reduces power consumption by limiting background activity, both when plugged in and on battery power. | — |
| WB.01.05.o02 | Energy saver | Control type not stated |  | — | Activates when the battery is low; dims the screen and limits background processes. | The page says it activates when the battery is low. |
| WB.01.05.o03 | Lower screen brightness when using energy saver | Control type not stated |  | — | Can be turned on to lower screen brightness when using energy saver. | Applies when using energy saver. |
| WB.01.05.o04 | Turn my screen off after | Dropdown | Times are selectable; specific choices are not stated. | Modern standby: on battery, 3 minutes; plugged in, 5 minutes. S3: on battery, 3 minutes; plugged in, 5 minutes. | Sets how long the device waits before turning off the screen when it is not in use. | Values differ by whether the device is on battery or plugged in. |
| WB.01.05.o05 | Make my device sleep after | Dropdown | Times are selectable; specific choices are not stated. | Modern standby: on battery, 3 minutes; plugged in, 5 minutes. S3: on battery, 10 minutes; plugged in, 15 minutes. | Sets how long the device waits before going to sleep when it is not in use. | Values differ by whether the device is on battery or plugged in. |

- SKU / hardware / version conditions: Screen and sleep defaults are listed separately for modern standby devices and S3 devices.

Preview/independent evidence (26H2-era):

- [Official documentation (T1)](https://learn.microsoft.com/en-us/windows-hardware/design/component-guidelines/energy-saver) — applies to: Windows 11 24H2 and later versions.
  - Energy saver (Windows Settings; the page path is not stated.): Users can configure Energy saver to run automatically when the device reaches a certain battery percentage, or to always run when the device is plugged in. The percentage choices are not stated.. When enabled, Windows manages power usage. Users cannot change the power mode in Windows Settings. Display brightness is reduced by 30%, and transparency effects on window backgrounds are disabled.
  - Display brightness (Setting associated with Energy saver; exact UI label and location are not stated.): Enable or disable the brightness reduction; no specific choices are stated.. When Energy saver is on, display brightness is reduced by 30%.
  - Requirement/rollout: Energy saver replaces battery saver and is available on Windows 11 24H2 and later versions.
  - Requirement/rollout: The page states no Copilot+ PC, Insider channel, region, or SKU requirement.
- [Independent (T4)](https://techdows.com/how-to-customize-windows-11-battery-settings.html) — applies to: Windows 11; the page says Energy saver replaced Battery saver in version 24H2. No build number is stated..
  - Turn energy saver on when battery level is at (Battery-level selector under Energy saver): Choose the battery percentage at which Energy saver turns on; specific choices are not stated.. Turns Energy saver on automatically when the battery reaches the chosen level.
  - Lower screen brightness when using energy saver (Toggle under Energy saver): On or off. Lowers screen brightness while Energy saver is in use.
  - Always use energy saver (Toggle under Energy saver): On or off. When turned on, keeps Energy saver enabled until the PC is plugged in.
  - Choose what closing the lid does (Control Panel power option, not a control on the Settings page): For On battery, the page recommends Sleep or Hibernate. It says separate On battery and Plugged in values are available.. Sets the action taken when the laptop lid is closed.
  - Requirement/rollout: No Copilot+ PC, Insider channel, region, or SKU requirement is stated.
  - Requirement/rollout: Energy saver is named Battery saver before Windows 11 version 24H2, according to the page.


### WB.01.05.01 — System > Power & battery > Energy saver

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > System > Power & battery > Energy saver`; `Settings > System > Power & battery > Energy saver`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/experience/power-battery/battery-saving-tips-for-windows); [Official support article (T1)](https://support.microsoft.com/en-gb/windows/battery-saving-tips-for-windows-a850d64d-ee8e-c8d2-6c75-8ffe6ea3ea99); [Official support article (T1)](https://support.microsoft.com/en-us/windows/experience/power-battery/power-settings-in-windows-11)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.05.01.o01 | Turn energy saver on when battery level is at | Dropdown | Battery level you'd like; specific choices are not listed. | — | Turns Energy saver on when the battery reaches the selected level. | — |
| WB.01.05.01.o02 | Always use energy saver | Toggle | On | — | When set to On, turns on Energy saver now and leaves it on until the next time you plug in your PC. | — |
| WB.01.05.01.o03 | Energy saver | Setting; control type not stated |  | — | Activates when battery is low; dims the screen and limits background processes. | The page says it activates when the battery is low. |
| WB.01.05.01.o04 | Lower screen brightness when using energy saver | Toggle; the page says to turn it on |  | — | Lowers screen brightness while Energy saver is in use. | Applies when using Energy saver. |

- Keyboard shortcuts: Select the Battery icon in the notification area, then select the Energy saver quick setting to turn it on.
- Windows 10 differences (legacy): The Windows 10 section describes Battery saver, not Energy saver. Its path is Start > Settings > System > Battery > Turn battery saver on automatically if my battery falls below:. To turn it on until the next charge, turn on Battery saver status until next charge.; Windows 10 uses Battery saver rather than Energy saver. The page gives the path Start > Settings > System > Battery and names the setting “Turn battery saver on automatically if my battery falls below:” and the toggle “Battery saver status until next charge”. It also says Battery saver temporarily turns off some power-intensive features, including automatic email and calendar syncing, live tile updates, and apps you aren’t actively using.


### WB.01.05.02 — System > Power & battery > Screen, sleep & hibernate timeouts

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Settings > System > Power & battery > Screen, sleep & hibernate timeouts`; `Start > Settings > System > Power & battery > Screen, sleep, & hibernate timeouts`; `Start > Settings > System > Power & battery > Screen, sleep & hibernate timeouts`

Sources: [Official support article (T1)](https://support.microsoft.com/topic/26f623b5-4fcc-4194-863d-b824e5ea7679); [Official support article (T1)](https://support.microsoft.com/en-us/windows/experience/power-battery/power-settings-in-windows-11); [Official support article (T1)](https://support.microsoft.com/en-gb/windows/power-settings-in-windows-11-0d6a2b6b-2e87-4611-9980-ac9ea2175734)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.05.02.o01 | Turn my screen off after | Dropdown | Time choices are not specified. | New defaults for modern standby devices: 3 minutes on battery; 5 minutes when plugged in. New defaults for S3 devices: 3 minutes on battery; 5 minutes when plugged in. | Sets how long the device waits before turning off the screen when you are not using it. | The setting applies both on battery power and when plugged in; defaults vary by device type. |
| WB.01.05.02.o02 | Make my device sleep after | Dropdown | Time choices are not specified. | New defaults for modern standby devices: 3 minutes on battery; 5 minutes when plugged in. New defaults for S3 devices: 10 minutes on battery; 15 minutes when plugged in. | Sets how long the device waits before going to sleep when you are not using it. | The setting applies both on battery power and when plugged in; defaults vary by device type. |
| WB.01.05.02.o03 | See more about how to shut down, sleep, or hibernate your PC | Link |  | — | The page names this link in the screen and sleep settings section. | — |

- SKU / hardware / version conditions: No Windows 10-specific differences are stated.; The article gives separate default screen and sleep timeout values for modern standby devices and S3 devices.; The page gives different default screen and sleep timeout values for modern standby devices and S3 devices.; The stated defaults differ for modern standby devices and S3 devices.


### WB.01.06 — System > Storage

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > System > Storage`; `Start > Settings > System > Storage > Cleanup recommendations`; `Start > Settings > System > Storage > Advanced storage settings > Where new content is saved`

ms-settings URIs: `ms-settings:storagesense`, `ms-settings:storagepolicies`

Sources: [Official support article (T1)](https://support.microsoft.com/en-gb/help/12425/manage-drive-space-with-storage-sense)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.06.o01 | Storage Sense | Not specified |  | — | Can automatically free up drive space by getting rid of items you don't need, such as temporary files and items in your Recycle Bin. | — |
| WB.01.06.o02 | Manage drive space with Storage Sense | Link |  | — | Opens information about managing drive space with Storage Sense. | — |
| WB.01.06.o03 | Cleanup recommendations | Page | Temporary files; Large or unused files; Files synced to the cloud; Unused apps | — | Review categories, select items to remove, and Windows will suggest files and show the amount of space you'll save. Select the Clean up button for each section. | The previous Windows version may appear as a system file if it has been fewer than 10 days since upgrading. Deleting it removes Windows.old, cannot be undone, and prevents going back to the previous Windows version. |
| WB.01.06.o04 | Temporary files | Category |  | — | Review and select items to remove as part of Cleanup recommendations. | — |
| WB.01.06.o05 | Large or unused files | Category |  | — | Review and select items to remove as part of Cleanup recommendations. | — |
| WB.01.06.o06 | Files synced to the cloud | Category |  | — | Review and select items to remove as part of Cleanup recommendations. | — |
| WB.01.06.o07 | Unused apps | Category |  | — | Review and select items to remove as part of Cleanup recommendations. | — |
| WB.01.06.o08 | Clean up | Button |  | — | Cleans up the selected items in its section. | Available after selecting items in a Cleanup recommendations section. |
| WB.01.06.o09 | Advanced storage settings | Section |  | — | Contains the Where new content is saved setting. | — |
| WB.01.06.o10 | Where new content is saved | Page with dropdown menus | A drive can be selected for each content type. | — | Sets another drive for new content instead of the default drive. | If the desired drive is not listed, make sure it is connected to the PC. |
| WB.01.06.o11 | Open Storage settings | Link |  | — | Opens Storage settings. | — |

- Windows 10 differences (legacy): The Windows 10 section describes Disk Cleanup as an alternative when the system doesn't have Storage Sense; it does not describe Cleanup recommendations for Windows 10.; Windows 10 uses Start > Settings > System > Storage > More storage settings > Change where new content is saved, rather than the Windows 11 path under Advanced storage settings.


### WB.01.06.01 — System > Storage > Cleanup recommendations

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > System > Storage > Cleanup recommendations`

ms-settings URIs: `ms-settings:storagerecommendations`

Sources: [Official support article (T1)](https://support.microsoft.com/help/12425)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.06.01.o01 | Temporary files | Category; select items to remove |  | — | Review the category and select items you'd like to remove. Windows suggests files and shows how much space you'll save. Select **Clean up** for the section to remove selected items. | — |
| WB.01.06.01.o02 | Large or unused files | Category; select items to remove |  | — | Review the category and select items you'd like to remove. Windows suggests files and shows how much space you'll save. Select **Clean up** for the section to remove selected items. | — |
| WB.01.06.01.o03 | Files synced to the cloud | Category; select items to remove |  | — | Review the category and select items you'd like to remove. Windows suggests files and shows how much space you'll save. Select **Clean up** for the section to remove selected items. | — |
| WB.01.06.01.o04 | Unused apps | Category; select items to remove |  | — | Review the category and select items you'd like to remove. Windows suggests files and shows how much space you'll save. Select **Clean up** for the section to remove selected items. | — |
| WB.01.06.01.o05 | Clean up | Button |  | — | Removes the selected items for its section. | Select items to remove in a category first. |

- Confirmations / errors: If fewer than 10 days have passed since upgrading Windows, the previous version of Windows may appear as a system file that can be deleted. Deleting it removes the Windows.old folder and cannot be undone; you will not be able to go back to the previous version of Windows.
- Windows 10 differences (legacy): The Windows 10 section does not describe Cleanup recommendations; it describes using Disk Cleanup to delete temporary and system files.


### WB.01.07 — System > Nearby sharing

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > System > Nearby sharing`

ms-settings URIs: `ms-settings:crossdevice`

Sources: [Official support article (T1)](https://support.microsoft.com/en-US/Windows/Experience/Connectivity-Networking/share-things-with-nearby-devices-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.07.o01 | Nearby sharing | Selection (control type not specified) | Everyone nearby; only your own devices (where you are logged in with your Microsoft account). | — | Chooses which devices you can share or receive content from. If Everyone nearby is chosen, the Bluetooth MAC address and name of your PC are made available to nearby devices; this info could be used to link your PC to you. | The page says to configure this on both the device sharing from and the device sharing to. |
| WB.01.07.o02 | Save files I receive to | Save-location setting | A new location can be chosen. | — | Specifies where received files are stored. | Use Change, choose a new location, then select Select folder. |
| WB.01.07.o03 | Change | Button |  | — | Opens the flow to choose a new location for received files. | Appears under Save files I receive to. |
| WB.01.07.o04 | Select folder | Button |  | — | Confirms the chosen folder for received files. | Used after choosing a new location. |

- Windows 10 differences (legacy): The Windows 10 Settings path is Start > Settings > System > Shared experiences.; The Windows 10 instructions also say to turn on Nearby sharing from action center on both devices.; The Windows 10 setting is labeled “I can share or receive content from”; it lets you choose which devices can share or receive content. The page also describes Save files I receive to, Change, and Select folder for Windows 10.


### WB.01.08 — System > Multitasking

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > System > Multitasking`

ms-settings URIs: `ms-settings:multitasking`, `ms-settings:multitasking-sgupdate`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/make-it-easier-to-focus-on-tasks-0d259fd9-e9d0-702c-c027-007f0e78eaf2)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.08.o01 | Snap windows | Toggle and expandable dropdown menu | The page says to turn the toggle on and select a desired option from the dropdown menu; it does not list the dropdown choices. | — | Automatically snaps windows for resizing and arranging them into layouts. | — |
| WB.01.08.o02 | 20 most recent tabs; 5 most recent tabs; 3 most recent tabs; Don't show tabs | Dropdown menu | 20 most recent tabs; 5 most recent tabs; 3 most recent tabs; Don't show tabs | — | Controls whether tabs from apps are displayed when snapping or pressing Alt+Tab. | — |
| WB.01.08.o03 | Desktops | Expandable group |  | — | Expands desktop display options. | — |
| WB.01.08.o04 | Only on the desktop I'm using; On all desktops | Dropdown menu | Only on the desktop I'm using; On all desktops | — | Controls which open windows appear on the taskbar. | Located under Desktops. |
| WB.01.08.o05 | Title bar window shake | Toggle | On or off | — | When turned on, quickly minimizes all open windows except the one you're working on. | — |
| WB.01.08.o06 | Help with Multitasking | Expandable section |  | — | Provides links for more information. | — |
| WB.01.08.o07 | Snap your windows | Link |  | — | Opens information about splitting the screen. | Located under Help with Multitasking. |
| WB.01.08.o08 | Windows 11 Multitasking Mastery | Link |  | — | Opens information about Task View. | Located under Help with Multitasking. |

- Keyboard shortcuts: Alt+Tab
- Windows 10 differences (legacy): The Settings path is written as Start > Settings > System > Multi-tasking.; The expandable group is labeled Virtual desktops, with dropdown choices Only on the desktop I'm using and All desktops for taskbar windows and Alt+Tab windows.; The links are labeled Splitting My Screen and Using Task View.; The page describes a Snap windows toggle and checkboxes to customize preferences, rather than the Windows 11 Snap windows dropdown described in the Windows 11 section.; Windows 10 includes a Timeline toggle.


### WB.01.08.01 — System > Multitasking > Desktops

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Settings > System > Multitasking > Desktops`

Sources: [Official support article (T1)](https://support.microsoft.com/en-US/Windows/Experience/configure-multiple-desktops-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.08.01.o01 | Desktops | Expandable section |  | — | Expand this section to access the desktop settings. | Located under System > Multitasking. |
| WB.01.08.01.o02 | On the taskbar, show all the open windows | Control type not stated | Only on the desktop I'm using; On all desktops | — | Configures whether the taskbar shows windows from only the current desktop or from all desktops. | Under Desktops. |
| WB.01.08.01.o03 | Show all open windows when I press Alt+Tab | Control type not stated | Open windows from all desktops; open windows only from the desktop you're currently using | — | Chooses whether Alt+Tab displays open windows from all desktops or only the current desktop. | Under Desktops. |
| WB.01.08.01.o04 | Open multitasking settings | Link |  | — | Opens multitasking settings. | — |

- Keyboard shortcuts: Alt+Tab


### WB.01.09 — System > Advanced (For developers)

Status: COMPLETE. Official Advanced settings page enumerates the full page contents (updated 2026-02-26); live-build check pending (VP-01).

Navigation paths (as stated in sources): `System > Advanced > For developers`; `For developers`

ms-settings URIs: `ms-settings:developers`

Sources: [Official documentation (T1)](https://learn.microsoft.com/en-us/windows/advanced-settings/developer-mode) (dated 2026-06-03)

Full page inventory from [Advanced Windows Settings](https://learn.microsoft.com/en-us/windows/advanced-settings/) (updated 2026-02-26). This is the basis for the COMPLETE status:

| # | Group | Setting (verbatim) | Description (verbatim) | Condition |
|---|---|---|---|---|
| WB.01.09.a01 | Taskbar | End Task | Enable end task in taskbar by right click | — |
| WB.01.09.a02 | File Explorer | File Explorer | Adjust settings for a more developer-friendly experience using File Explorer | — |
| WB.01.09.a03 | File Explorer | ↳ Show file extensions | Show file extensions in File Explorer | — |
| WB.01.09.a04 | File Explorer | ↳ Show hidden and system files | Show hidden and system files in File Explorer | — |
| WB.01.09.a05 | File Explorer | ↳ Show full path in title bar | Show the full path in the File Explorer title bar | — |
| WB.01.09.a06 | File Explorer | ↳ Show option to run as different user in Start | Show option to run as a different user in the Start menu | — |
| WB.01.09.a07 | File Explorer | ↳ Show empty drives | Show empty drives in File Explorer | — |
| WB.01.09.a08 | File Explorer | File Explorer + version control | View branch, commit, and author details directly in File Explorer | Windows Insider Beta Channel only |
| WB.01.09.a09 | File Explorer | Enable long paths | Remove MAX_PATH limitations from common Win32 file and directory functions | — |
| WB.01.09.a10 | Virtual Workspace | Remote Desktop | Enable Remote Desktop and ensure machine availability | — |
| WB.01.09.a11 | Virtual Workspace | Virtual Workspaces | Adjust settings for virtual workspaces | — |
| WB.01.09.a12 | Terminal | Terminal | Choose the default terminal app to host command-line apps | — |
| WB.01.09.a13 | Terminal | PowerShell | Turn on these settings to execute PowerShell scripts | — |
| WB.01.09.a14 | Terminal | Enable sudo | Enable the sudo command | Windows 11 24H2+ |
| WB.01.09.a15 | For developers | Developer Mode | Install apps from any source, including loose files | Requires local administrator |
| WB.01.09.a16 | For developers | Device Portal | Turn on remote diagnostics over local area network connections | — |
| WB.01.09.a17 | Dev Drive | Dev Drive | A storage volume optimized for performance in developer scenarios | — |

Troubleshooting (documented): managed devices may have policy-disabled toggles; Developer Mode requires admin; preview features may require an Insider channel.


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.09.o01 | Developer Mode setting | Toggle | Enabled or disabled | — | Enabling Developer Mode provides sideloading, debugging, and additional deployment options. On desktop, it installs a package of features. Disable Developer Mode to stop its SSH service. | Requires administrator access. The option may be disabled on organization-owned devices. Read the disclaimer and select Yes to accept the change. |
| WB.01.09.o02 | Enable Device Portal | Option | On or off | — | Enables Device Portal; firewall rules for Device Portal are configured only when this option is turned on. | Device Portal is among the features installed when Developer Mode is enabled on desktop. |
| WB.01.09.o03 | Device Discovery | Option | Enabled or disabled | — | Makes the device visible to other devices on the network through mDNS and enables the SSH server. Turning it off turns off the Developer Mode SSH services. | Enable it when the device is intended to be a remote deployment target. The SSH services are enabled when Device Discovery is enabled. |
| WB.01.09.o04 | Pair | Button |  | — | Displays the SSH PIN used for pairing. The PIN prompt must be displayed to complete the first Visual Studio deployment targeting the device; the PIN is valid only while displayed. | The button is exposed immediately after Device Discovery is enabled. |
| WB.01.09.o05 | settings for developers | Link |  | — | Opens the System > Advanced settings page. | Shown in the Visual Studio dialog when a Windows project is built without Developer Mode enabled. |
| WB.01.09.o06 | Yes | Button |  | — | Accepts the Developer Mode change after the disclaimer is displayed. | Shown when enabling Developer Mode. |

- Confirmations / errors: Enabling Developer Mode requires administrator access; the option may be disabled if the device is owned by an organization.; For a domain-joined computer, the page advises consulting the network administrator because the Developer Mode package may be blocked by WSUS settings.; If the Developer Mode package does not install correctly, remote deployment to this PC using Device Portal or Device Discovery to enable SSH is unavailable; local deployment using Visual Studio remains possible.; The page describes package errors for failure to locate the package and failure to install it, both with error code 0x80004005. Suggested checks include internet connectivity, Windows updates, and whether the Windows Developer Mode package is available under Optional features.; The page says Device Portal and Device Discovery are useful when developing on one machine and deploying to another for testing.
- Restart / sign-out: For a Developer Mode package installation failure, the page says to restart the computer to ensure updates are applied.
- SKU / hardware / version conditions: On desktop, enabling Developer Mode installs a package of features.; The page says the group policy method is unavailable on Windows 10 Home and Windows 11 Home; those editions require registry or PowerShell methods instead. These methods do not enable the SSH server or allow remote deployment and debugging.
- Windows 10 differences (legacy): Before Windows 11 25H2, these settings appear on the For developers page; in Windows 11 25H2 and later, they appear in the For developers section of the Advanced settings page.; For Windows versions older than Windows 10 22H2, the page gives the Optional features path as Settings > Apps > Apps & features > Optional features > Add a feature. For newer versions, it gives Settings > System > Optional features > Add a feature.

Preview/independent evidence (26H2-era):

- [Independent (T4)](https://www.windowscentral.com/microsoft/windows-11/settings-apps-best-features-available-in-2025-on-windows-11-version-25h2-and-24h2) — applies to: Windows 11 versions 25H2 and 24H2; no specific build is stated..
  - Advanced (Page replacing "For Developers"; groups settings into different sections.): Not stated — default: Not stated. Contains the previous settings and introduces new features.
  - File Explorer (Page within Advanced with a version control feature.): Not stated — default: Not stated. Integrates File Explorer with Git.
  - Requirement/rollout: The article says most changes are available in both Windows 11 versions 25H2 and 24H2.
  - Requirement/rollout: The article cautions that Microsoft may still be finishing or rolling out changes; it gives no specific Insider channel, region, SKU, or Copilot+ PC requirement for Advanced settings.
- [Official documentation (T1)](https://learn.microsoft.com/en-us/windows/advanced-settings/) — applies to: Windows 11, version 25H2 and later: the former For developers settings are surfaced under Advanced. Sudo for Windows is available in Windows 11, version 24H2 and later..
  - End Task (Taskbar setting). Enable end task in taskbar by right click.
  - File Explorer (File Explorer settings category). Adjust settings for a more developer-friendly experience using File Explorer.
  - Show file extensions (File Explorer setting). Show file extensions in File Explorer.
  - Show hidden and system files (File Explorer setting). Show hidden and system files in File Explorer.
  - Show full path in title bar (File Explorer setting). Show the full path in the File Explorer title bar.
  - Show option to run as different user in Start (File Explorer setting). Show option to run as a different user in the Start menu.
  - Show empty drives (File Explorer setting). Show empty drives in File Explorer.
  - File Explorer + version control (File Explorer setting). View branch, commit, and author details directly in File Explorer.
  - Enable long paths (File Explorer setting). Remove MAX_PATH limitations from common Win32 file and directory functions.
  - Remote Desktop (Virtual Workspace setting). Enable Remote Desktop and ensure machine availability.
  - Virtual Workspaces (Virtual Workspace setting). Adjust settings for virtual workspaces.
  - Terminal (Terminal setting). Choose the default terminal app to host command-line apps.
  - PowerShell (Terminal setting). Turn on these settings to execute PowerShell scripts.
  - Enable sudo (Terminal setting). Enable the sudo command.
  - Developer Mode (For developers setting). Install apps from any source, including loose files.
  - Requirement/rollout: In Windows 11, version 25H2 and later, the former For developers settings are surfaced under the Advanced page.
  - Requirement/rollout: File Explorer version control integration is currently available via the Windows Insider Beta Channel.
  - Requirement/rollout: Enabling Developer Mode requires local administrator privileges.
  - Requirement/rollout: Devices managed by an organization may have policies that disable Advanced settings toggles.
  - Requirement/rollout: Ensure the device is updated to a supported Windows version; preview features may require joining a Windows Insider channel.


### WB.01.10 — System > Activation

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > System > Activation`

ms-settings URIs: `ms-settings:activation`

Sources: [Official support article (T1)](https://support.microsoft.com/en-US/windows/activation/product-activation-for-windows-online-microsoft-support-product-activation-portal)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.10.o01 | Activation | Settings page |  | — | Opens the Windows Activation settings page, where you can review activation state and enter a product key if needed. | — |
| WB.01.10.o02 | Open activation settings | Button or link |  | — | Opens Activation settings. | — |
| WB.01.10.o03 | Automated Help | Option |  | — | On the Activation page, opens the flow that displays the Installation ID. | For Windows 11 builds 26200.8973, 26100.8973, and newer. |
| WB.01.10.o04 | Activate by Phone | Option |  | — | Opens the flow to select a Country or Region and display the Installation ID. | For earlier Windows 11 builds. |
| WB.01.10.o05 | Country or Region | Selection |  | — | Select this before proceeding to display the Installation ID. | Used in the earlier-build offline activation flow. |
| WB.01.10.o06 | Next | Button |  | — | Proceeds after selecting a Country or Region. | Used in the earlier-build offline activation flow. |
| WB.01.10.o07 | Enter Confirmation ID | Button or option |  | — | Lets you enter the Confirmation ID received through the Product Activation Portal. | After obtaining a Confirmation ID. |
| WB.01.10.o08 | Activate Windows | Button |  | — | Submits the entered Confirmation ID to complete activation. | After entering a Confirmation ID. |
| WB.01.10.o09 | Activate | Button |  | — | Activates Windows after entering a Confirmation ID. | The page uses this label in the Windows 8.1/8 flow, not the Windows 11 flow. |
| WB.01.10.o10 | product key | Entry field |  | — | Enter the product key if needed. | The page does not state that a product key is always required. |

- Confirmations / errors: For offline activation, the article says not to call any activation phone number that may be displayed; use the Product Activation Portal instead.; If activation still fails, the article directs readers to Windows activation troubleshooting and activation error help.; If online activation fails, or there is no internet connection, the article describes offline activation using an Installation ID and Confirmation ID.; The Activation page says to review the activation state; the page does not state a particular default state.; The article says Windows automatically attempts online activation when connected to the internet.
- Keyboard shortcuts: Windows logo key (opens the Start menu)
- SKU / hardware / version conditions: For Windows 11 builds 26200.7922 and newer, the article says SLUI may no longer provide the expected activation experience or be available.; For earlier Windows 11 builds, Method 1 says to use Activate by Phone.; Windows 11 Method 1 uses Automated Help on builds 26200.8973, 26100.8973, and newer.
- Windows 10 differences (legacy): For offline activation, Windows 10 uses Activate by Phone to display the Installation ID.; The Windows 10 Product Activation Portal selection is Activate a Microsoft Product > Windows > Windows 10.; The Windows 10 navigation path is Start > Settings > Update & Security > Activation.


### WB.01.11 — System > Troubleshoot

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > System > Troubleshoot > Other troubleshooters`

ms-settings URIs: `ms-settings:troubleshoot`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/deployment/updates-lifecycle/troubleshoot-problems-updating-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.11.o01 | Troubleshoot | Settings page |  | — | Open Troubleshoot from System settings. | Windows 11 |
| WB.01.11.o02 | Other troubleshooters | Page or section |  | — | Choose this to locate the Windows Update troubleshooter. | Windows 11 |
| WB.01.11.o03 | Windows Update | Troubleshooter entry |  | — | Select Run to start the troubleshooter; follow the on-screen instructions to complete the process. | Windows 11 |
| WB.01.11.o04 | Run | Button |  | — | Runs the Windows Update troubleshooter. | Windows 11 |

- Windows 10 differences (legacy): Windows 10 navigation is Start > Settings > Update & Security > Troubleshoot > Additional troubleshooters. Under Get up and running, select Windows Update, then Run the troubleshooter.


### WB.01.12 — System > Recovery

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `System > Recovery > Point-in-time restore`; `Recovery > Advanced startup`; `Settings > System > Recovery > Point-in-time restore`; `Settings > System > Recovery > Advanced startup > Restart now`; `WinRE > Troubleshoot > Point-in-time restore`

ms-settings URIs: `ms-settings:recovery`

Sources: [Official support article (T1)](https://support.microsoft.com/help/12415/reset-your-pc); [Official support article (T1)](https://support.microsoft.com/kb/307654/windows-recovery-environment); [Official support article (T1)](https://support.microsoft.com/en-us/windows/experience/backup-recovery/recovery-options-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.12.o01 | Point-in-time restore | Settings page option |  | — | View available restore points. If a restore point is available from before the issue occurred, the PC can be started in WinRE to select it. This rolls back the PC, including apps, settings, and personal files; changes made after the selected restore point are lost. | Windows 11, version 24H2 or later. |
| WB.01.12.o02 | Advanced startup | Settings page option |  | — | Used to boot into WinRE. | To use Point-in-time restore, navigate here and select Restart now. |
| WB.01.12.o03 | Restart now | Button |  | — | Restarts the PC into WinRE. | Selected from Advanced startup. |

- Confirmations / errors: Before recovery, back up important files and enable Windows Backup.; If BitLocker is enabled, the BitLocker recovery key is needed to access most recovery options from WinRE.; Point-in-time restore keeps restore points for up to 3 days.; Point-in-time restore loses changes made after the selected restore point. Files stored in OneDrive or another cloud service are not affected.
- Restart / sign-out: Restart now; Select Restart now under Advanced startup to boot into WinRE.; Select Restart now under Advanced startup to enter WinRE.
- SKU / hardware / version conditions: Point-in-time restore is available on Windows 11, version 24H2 or later.; Point-in-time restore requires Windows 11, version 24H2 or later.
- Windows 10 differences (legacy): Point-in-time restore is not listed as available for Windows 10.; Point-in-time restore is stated as available only on Windows 11, version 24H2 or later. System Restore is available on both Windows 10 and Windows 11.; Point-in-time restore requires Windows 11, version 24H2 or later; the page does not list it for Windows 10.; System Restore is available on both Windows 10 and Windows 11; it rolls back system files and settings, not personal files.


### WB.01.12.01 — System > Recovery > Fix problems using Windows Update

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > System > Recovery`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/deployment/install-upgrade/fix-issues-by-reinstalling-the-current-version-of-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.12.01.o01 | Fix problems using Windows Update | Recovery option/section |  | — | Reinstalls the same version of Windows, repairs system files and components, and preserves apps, files, and settings. | Available on supported Windows 11 PCs; may be unavailable on work or school-managed PCs or devices with updates managed. |
| WB.01.12.01.o02 | Open Recovery settings | Link |  | — | Opens Recovery settings. | — |
| WB.01.12.01.o03 | Reinstall now | Button |  | — | Opens the “Fix problems using Windows Update” window. The reinstall downloads and installs a repair version of the last operating system update successfully installed on the PC. | The option may be unavailable on some PCs; see the availability conditions. |
| WB.01.12.01.o04 |  | Choice in the “Fix problems using Windows Update” window | Allow the PC to automatically restart 15 minutes after the installation completes, or do not allow it to restart automatically. | — | Determines whether the PC restarts automatically after installation. | — |
| WB.01.12.01.o05 | OK | Button |  | — | Starts the installation when selected. | — |

- Confirmations / errors: If an update fails to install because of problems related to system files or components, Windows Update may display: “Reinstall your current version of Windows to repair system files and components.”; Keep the PC connected to a power source and the internet during the entire reinstallation process.
- Restart / sign-out: If automatic restart is not selected, restart the PC after the reinstall completes.; The PC can be set to restart automatically 15 minutes after installation completes.
- SKU / hardware / version conditions: Applies to Windows 11.; It is unavailable on devices whose updates are managed, including devices enrolled in Autopatch, using the deployment service, or configured with the listed update policies.; The PC also needs the February 2024 optional update, or a later update.; The option might be missing on a version of Windows earlier than Windows 11 with the 2022 annual update.; The option might be unavailable if the PC is managed by work or school.


### WB.01.12.02 — System > Recovery > Point-in-time restore

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Settings > System > Recovery`; `Settings > System > Recovery > Point-in-time restore`; `Settings > System > Recovery > Point-in-time restore > View or edit`

Sources: [Official support article (T1)](https://support.microsoft.com/en-US/Windows/experience/backup-recovery/point-time-restore-for-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.12.02.o01 | View or edit | Button |  | — | Opens the configuration view for point-in-time restore. | Only local admins can view or edit configurations. |
| WB.01.12.02.o02 | On / Off | Toggle | On; Off | On* | When turned on, point-in-time restore automatically creates restore points. When turned off, existing restore points remain until they expire or disk space is needed. | The asterisk states that the feature is currently disabled by default on IT-managed devices. It is already enabled by default on unmanaged devices, and starting with Windows 11, version 26H2, it will be enabled by default on IT-managed devices. For the feature to be enabled by default, the Windows OS volume must be 200 GB or larger; devices with smaller OS volumes can enable it manually. |
| WB.01.12.02.o03 | Restore point frequency | Setting |  | Every 24 hours** | Sets how often restore points are created. | Configurable only on Windows devices running the Enterprise edition. |
| WB.01.12.02.o04 | Restore point retention | Setting |  | 72 hours** | Sets how long restore points are kept before automatic deletion. | Configurable only on Windows devices running the Enterprise edition. |
| WB.01.12.02.o05 | Maximum usage limit | Setting |  | 2% of disk | Sets the upper limit for total disk space used by restore points. Space is not reserved in advance; unused capacity remains available to the system. | — |

- SKU / hardware / version conditions: For point-in-time restore to be enabled by default, the Windows OS volume must be 200 GB or larger. Devices with smaller OS volumes can enable it manually.; On IT-managed devices, point-in-time restore is currently disabled by default; starting with Windows 11, version 26H2, it will be enabled by default. It is already enabled by default on unmanaged devices.; Only local admins can view or edit configurations for point-in-time restore.; Restore point frequency and Restore point retention are configurable only on Windows devices running the Enterprise edition.

Preview/independent evidence (26H2-era):

- [Official documentation (T1)](https://learn.microsoft.com/en-gb/windows/configuration/quick-machine-recovery/) — applies to: Windows 11, version 24H2 with KB5062660.
  - Quick machine recovery (Toggle): On; the page says to turn the toggle to On to enable quick machine recovery. — default: Not stated for this Settings toggle.. Enables quick machine recovery.
  - Retry and restart intervals (Intervals to configure): Configure as needed; specific Settings-page choices are not stated. — default: Not stated.. Sets the retry and restart intervals for recovery.
  - Enable Cloud Remediation (Intune Settings catalog setting): Enable or disable cloud remediation. — default: Enabled by default on Windows Home; disabled by default on Windows Pro and Enterprise.. When enabled, the device uses Windows Update during recovery; when disabled, Windows uses local Startup Repair.
  - Enable Auto Remediation (Intune Settings catalog setting): Enable or disable auto remediation. — default: Turned off by default on Windows Home; disabled by default on Windows Pro and Enterprise.. When enabled, the device automatically tries to find a remediation and retries if none is found on the first attempt. When disabled or not configured, manual intervention is required.
  - Requirement/rollout: The page applies to Windows 11, version 24H2 with KB5062660.
  - Requirement/rollout: Supported recovery network connections are wired networks and WPA/WPA2 password-based Wi-Fi.
  - Requirement/rollout: The page does not state a Copilot+ PC, Insider channel, or region requirement.
  - Requirement/rollout: The page gives different cloud-remediation and auto-remediation defaults by Windows edition; it does not state a separate rollout requirement.
- [Official documentation (T1)](https://learn.microsoft.com/ja-jp/windows/configuration/quick-machine-recovery/?tabs=settings) — applies to: Windows 11 バージョン 24H2 以降、KB5062660。.
  - クイック マシンの回復 (トグル): オン／オフ — default: Windows Home ではクラウド修復が既定で有効。Windows Pro と Enterprise では既定で無効。. クイック マシンの回復を有効にするには、トグルをオンにする。
  - クラウド修復 (設定項目): 有効／無効 — default: Windows Home では既定で有効。Windows Pro と Enterprise では既定で無効。. 有効にすると、復旧時にネットワークへ接続して Windows Update から修復を検索する。無効の場合は、ローカルの回復オプションとしてスタートアップ修復を使用する。
  - 自動修復 (設定項目): 有効／無効 — default: Windows Home、Pro、Enterprise では既定でオフまたは無効。. 有効にすると、Windows Update への接続と修復検索を自動で試み、解決策が見つからない場合は再試行する。無効または未構成の場合、回復の続行に手動操作が必要。
  - 再試行間隔 (間隔設定): 1～4320 分（既定値 30 分） — default: 30 分. 自動修復中に修復検索を再試行する間隔を設定する。
  - 再起動する時間 (時間設定): 1～4320 分（既定値 180 分） — default: 180 分. 自動修復中、再起動するまでの時間を設定する。最大 72 時間。
  - Requirement/rollout: 記事の適用対象は Windows 11。機能についてはバージョン 24H2 以降、KB5062660 と記載。
  - Requirement/rollout: Home、Pro、Enterprise の既定値が記載されている。
  - Requirement/rollout: Copilot+ PC、Insider チャネル、地域に関する要件は記載されていない。


### WB.01.13 — System > Projecting to this PC

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > System > Projecting to this PC`; `Projecting to this PC > Optional features > Add an optional feature > View features`

ms-settings URIs: `ms-settings:project`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/hardware/display-graphics/screen-mirroring-and-projecting-to-your-pc-or-wireless-display?nochrome=true&shownav=false)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.13.o01 | Add the “Wireless Display” optional feature to project to this PC | Section label | Select Optional features to add Wireless Display. | — | Opens the optional-features flow. | — |
| WB.01.13.o02 | Optional features | Settings link |  | — | Opens the optional-features flow for adding Wireless Display. | Shown under “Add the “Wireless Display” optional feature to project to this PC”. |
| WB.01.13.o03 | Add an optional feature | Button or control |  | — | Select View features, then search for “wireless display.” | Windows 11 instructions. |
| WB.01.13.o04 | View features | Button |  | — | Opens the list of optional features to search. | Next to “Add an optional feature”. |
| WB.01.13.o05 | Wireless Display | Checkbox | Checked or unchecked | — | Select the checkbox, then select Next and Install. | Select it from the search results for “wireless display”. |
| WB.01.13.o06 | Next | Button |  | — | Continues the Wireless Display installation flow. | After selecting Wireless Display. |
| WB.01.13.o07 | Install | Button |  | — | Installs Wireless Display. | After selecting Next. |
| WB.01.13.o08 | Back | Navigation control |  | — | Returns to the Projecting to this PC page after Wireless Display is installed. | After installation. |
| WB.01.13.o09 | Projecting to this PC | Settings page |  | — | The page to return to after installation. The article does not list its projecting preferences or their controls. | After Wireless Display is installed. |

- Confirmations / errors: If the PC you want to project to cannot be found, make sure it has Wi-Fi turned on and the wireless display app installed and launched.
- Keyboard shortcuts: Windows logo key + K — on the PC you’re projecting from, select the PC to connect to.
- SKU / hardware / version conditions: The Wireless Display app is available on devices running Windows 11, version 22H2 and later. For an earlier Windows version, launch the Connect app instead.; To find the PC you want to project to, it must have Wi-Fi turned on and the wireless display app installed and launched.
- Windows 10 differences (legacy): After installation, manage projecting preferences on the Projecting to this PC page in Settings, and launch the Connect app from taskbar search.; The Windows 10 instructions use Start > Settings > System > Projecting to this PC.; Under “Add the “Wireless Display” optional feature to project to this PC,” select Optional features, then Add a feature and search for “wireless display.” Select the Wireless Display checkbox and Install.


### WB.01.14 — System > Remote Desktop

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > System > Remote Desktop`

ms-settings URIs: `ms-settings:remotedesktop`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/help/4028379); [Official support article (T1)](https://support.microsoft.com/en-us/windows/experience/connectivity-networking/how-to-use-remote-desktop); [Official support article (T1)](https://support.microsoft.com/help/17463)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.14.o01 | Remote Desktop | Toggle |  | — | Turn it on, then select Confirm. | The remote PC must be running Windows 11 Pro. |
| WB.01.14.o02 | Confirm | Button |  | — | Select after turning on Remote Desktop. | Appears in the process of enabling Remote Desktop. |
| WB.01.14.o03 | PC name | PC name displayed on the Remote Desktop page |  | — | Make note of the PC name; you’ll need it to connect later. | Shown on the remote PC. |

- Confirmations / errors: After turning on Remote Desktop, select “Confirm.”; The page says to select Confirm after turning on Remote Desktop.
- SKU / hardware / version conditions: The client device can run any edition of Windows, including Pro or Home, or a different operating system.; The remote PC must be running Windows 11 Pro.
- Windows 10 differences (legacy): The Windows 10 instructions call the toggle “Enable Remote Desktop”; the Windows 11 instructions call it “Remote Desktop”.; The Windows 10 instructions say the edition must be Windows 10 Pro.; The Windows 10 remote PC must be running Windows 10 Pro.; The Windows 10 section names the toggle “Enable Remote Desktop” (the Windows 11 section names it “Remote Desktop”).; The Windows 10 toggle is labeled “Enable Remote Desktop”; the Windows 11 instructions refer to the toggle as “Remote Desktop.”


### WB.01.15 — System > Clipboard

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > System > Clipboard`

ms-settings URIs: `ms-settings:clipboard`

Sources: [Official support article (T1)](https://support.microsoft.com/windows/using-the-clipboard-30375039-ce71-9fe4-5b30-21b7aab6b13f)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.15.o01 | Clipboard history across your devices | Toggle | On | — | Turning it on enables clipboard syncing across devices. | Sync is tied to your Microsoft account or work account; use the same login information on all devices. |
| WB.01.15.o02 | Automatically sync text that I copy | Option |  | — | Automatically syncs copied text to other devices. | Clipboard history across your devices must be turned on. |
| WB.01.15.o03 | Manually sync text that I copy | Option |  | — | Requires opening clipboard history with Windows logo key + V and selecting the sync button on a clipboard item; the text is then uploaded to the cloud and synced to other devices. | Clipboard history across your devices must be turned on. |
| WB.01.15.o04 | Clear clipboard data | Settings area |  | — | Contains the Clear control for clearing clipboard data. | — |
| WB.01.15.o05 | Clear | Button |  | — | Clears clipboard data on the device and in the cloud, except for pinned items. | Located in the Clear clipboard data area. |

- Restart / sign-out: Clipboard history is cleared each time the PC restarts, except for pinned items.
- Keyboard shortcuts: Windows logo key + V
- Windows 10 differences (legacy): The sync toggle is labeled “Sync across devices” instead of “Clipboard history across your devices.”


### WB.01.16 — System > Optional features

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > System > Optional features`; `Start > Run > ms-settings:optionalfeatures`; `Settings > System > Optional features`

ms-settings URIs: `ms-settings:optionalfeatures`

Sources: [Official documentation (T1)](https://learn.microsoft.com/en-us/windows/client-management/client-tools/add-remove-hide-features?pivots=windows-11) (dated 2024-01-07)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.16.o01 | Optional features | Settings page |  | — | Opens the page for adding or removing optional features. | On supported Windows 11 versions, the page might instead be under Settings > Apps. |
| WB.01.16.o02 | View features | Button |  | — | Opens the Add an optional feature window. | Windows 11; next to Add an optional feature. |
| WB.01.16.o03 | Add an optional feature | Window | Available features; multiple features can be selected using the checkbox next to each feature. | — | After selecting features, select Next, review the selected list, and select Add to add them. | The device needs to be online so Windows Update can download the content needed to add features. |
| WB.01.16.o04 | Next | Button |  | — | Proceeds after selecting the desired features. | In the Add an optional feature window. |
| WB.01.16.o05 | Add | Button |  | — | Adds the selected features after the selected list is reviewed. | In the Add an optional feature window. |
| WB.01.16.o06 | Installed features | Section |  | — | Lists added features; search for a feature or scroll through the list to find it. | — |
| WB.01.16.o07 | Search installed features | Search box |  | — | Searches installed features for a feature to remove. | Under Installed features. |
| WB.01.16.o08 | Remove | Button |  | — | Removes the selected installed feature. | Select the feature to expand it, then select Remove. |

- Windows 10 differences (legacy): For Windows 10 removal, the search box is labeled Find an installed optional feature. Select the feature to expand it, then select Remove.; For some supported Windows 10 versions, the page might instead be under Settings > Apps > Apps & features.; The Windows 10 page uses a + button next to Add a feature to open the Add an optional feature window. Select feature checkboxes, then select Add.; Windows 10 navigation: Start > Settings > System > Optional features.


### WB.01.17 — System > About

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `設定 > 系統 > 關於`

ms-settings URIs: `ms-settings:about`

Sources: [Official support article (T1)](https://support.microsoft.com/zh-hk/windows/%E9%87%8D%E6%96%B0%E5%91%BD%E5%90%8D%E6%82%A8%E7%9A%84-windows-%E8%A3%9D%E7%BD%AE-750bc75d-8ff8-e99a-b9dc-04dff566ae74) (dated 2026-05-29)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.17.o01 | [重新命名此電腦] | 按鈕 |  | — | 選取後可輸入新名稱。 | — |
| WB.01.17.o02 | [下一步] | 按鈕 |  | — | 輸入新名稱後選取。 | — |
| WB.01.17.o03 | [立即重新啟動] | 按鈕 |  | — | 重新命名後立即重新啟動裝置。 | — |
| WB.01.17.o04 | [稍後重新啟動] | 按鈕 |  | — | 重新命名後稍後重新啟動裝置。 | — |

- Restart / sign-out: 重新命名後，可選取 [立即重新啟動] 或 [稍後重新啟動]。

Preview/independent evidence (26H2-era):

- [Independent (T4)](https://pureinfotech.com/windows-11-new-about-settings/) — applies to: Windows 11 builds 26220.6982 and 26120.6982.
  - This device (Option to rename the device): Not stated — default: Not stated. Shows a desktop thumbnail and the computer name.
  - Device info (Section displaying device specifications): Not stated — default: Not stated. Shows the same basic technical specifications as before, but without storage or graphics information.
  - Device insights (Section): Not stated — default: Not stated. Renamed from “Frequently Asked Questions” and separated from “Device info.”
  - Windows specifications (Information section under “This operating system”): Not stated — default: Not stated. Shows the same Windows details as before.
  - Storage (Entry point under “Related”): Not stated — default: Not stated. Provides access to Storage settings.
  - Requirement/rollout: The changes are available in Windows Insider builds 26220.6982 and 26120.6982 in the Dev and Beta Channels.
  - Requirement/rollout: The changes are still in the works through the Windows Insider Program; the article says it is unclear when they will become available for everyone.
  - Requirement/rollout: No Copilot+ PC, region, or SKU requirement is stated.


### WB.01.18 — System > Agent in Settings

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/experience/exploring-windows-settings); [Official support article (T1)](https://support.microsoft.com/en-gb/windows/exploring-windows-settings-56bdfb5a-b809-4e65-9c90-4bab76745e19); [Official support article (T1)](https://support.microsoft.com/en-us/windows/exploring-windows-settings-56bdfb5a-b809-4e65-9c90-4bab76745e19?nochrome=true)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.01.18.o01 | Agent in Settings | On-device AI agent | Natural-language requests, such as “My mouse pointer is too small” and “How can I share internet with other devices.” | — | Understands intent and can suggest fixes for common issues. The page says it can help change settings with your permission and at your initiation, and clarifies that it only suggests settings and makes no changes automatically. You must select **Apply** to enable or disable a setting; you can undo a change if needed. | Available on Copilot+ PCs. Language requirements: Chinese (Simplified), English, French, German, Hindi, Italian, Japanese, Korean, Portuguese, and Spanish. Available in all countries except China. |
| WB.01.18.o02 | Apply | Button | Enable or disable a suggested setting. | — | Applies the setting change; the agent does not make changes automatically. | Select it after the agent suggests a setting. |
| WB.01.18.o03 | undo | Undo action; control type not stated |  | — | Can be used to undo a change if needed. | Applies after a setting change. |

- Confirmations / errors: If the agent cannot confidently classify a query to a specific setting, standard search results are shown.; If the model cannot confidently classify a query to a specific setting, standard search results appear.; If the model cannot confidently classify a query to a specific setting, standard search results are shown.; The agent suggests settings; it does not make changes automatically.; The article describes “Agent in Settings” but does not identify it as a page under “System” or provide a Settings navigation path to it.
- Keyboard shortcuts: I (listed as the keyboard shortcut to open Settings).
- SKU / hardware / version conditions: Agent in Settings is available on Copilot+ PCs.; Geography: all countries except China.; Language and geography requirements apply: supported languages are Chinese (Simplified), English, French, German, Hindi, Italian, Japanese, Korean, Portuguese, and Spanish; all countries are supported except China.; Language requirements: Chinese (Simplified), English, French, German, Hindi, Italian, Japanese, Korean, Portuguese, and Spanish.; The Agent in Settings is available on Copilot+ PCs.; The Settings Mu model runs locally on the device and matches queries with relevant settings available in the Settings app.; The Settings Mu model runs locally on the device.

Preview/independent evidence (26H2-era):

- [Official Insider/Experience blog (T1, preview-scoped)](https://blogs.windows.com/windowsexperience/2025/10/16/new-experiences-currently-rolling-out-for-windows-11/) — applies to: Windows 11, versions 25H2 and 24H2. No specific build number is stated..
  - Agent in Settings (Agent search results). A direct navigation link is provided to the corresponding Settings page in agent search results.
  - Requirement/rollout: The article says the features are rolling out and advises installing the October 2025 security update to get the latest features and improvements.
  - Requirement/rollout: The article states that the improvements also apply to Windows 11, version 24H2, because versions 24H2 and 25H2 share a code base and servicing branch.
  - Requirement/rollout: Footnote 5 says inputs are initially limited to English and availability varies by market.
- [Official Insider/Experience blog (T1, preview-scoped)](https://blogs.windows.com/windowsexperience/2025/07/22/windows-11-is-the-home-for-ai-on-the-pc-with-even-more-experiences-available-today/) — applies to: Windows 11; no build number stated..
  - Agent in Settings (Settings search box): Enter a request in your own words, such as “I want to enable quiet hours” or “connect Bluetooth device.” For some settings, a specific request is supported, such as “change my resolution to 1920×1080.” — default: Not stated.. Offers recommendations and, when it can make the change, an option to complete the action with one click and an option to undo the change. If it cannot adjust the setting, improved Windows search surfaces relevant settings.
  - Requirement/rollout: The agent in Settings begins rolling out July 22, 2025, to Windows customers using English on Snapdragon-powered Copilot+ PCs.
  - Requirement/rollout: Availability on Intel- and AMD-powered devices was stated as of Aug. 29, 2025; the blog’s Sept. 5 editor’s note updates availability to reflect Intel and AMD silicon.
  - Requirement/rollout: Requires a Copilot+ PC. No Insider channel, Windows SKU, or specific Windows build is stated.


### WB.01.19 — System > System components

Status: PARTIAL. Only independent or preview-era evidence; no official Microsoft documentation found for this page.


Preview/independent evidence (26H2-era):

- [Independent (T4)](https://winaero.com/windows-11-has-got-a-new-system-components-page-in-settings/) — applies to: Windows 11 Build 25926 (Canary).
  - System Components (Settings page, opened from a button in the System section): List view or icon view — default: List view. Lists inbox apps, codecs, and plugins, with icons, installation date, app size, and extra options to view component options or remove a component.
  - Requirement/rollout: The page is described as a work in progress and hidden by default in Windows 11 Build 25926 (Canary).
  - Requirement/rollout: The article says to enable feature ID 44702824 with ViVeTool, then restart: `c:\vivetool\vivetool /enable /id:44702824`.
  - Requirement/rollout: No Copilot+ PC, region, or SKU requirement is stated.
- [Independent (T4)](https://www.windowslatest.com/2023/10/16/windows-11-23h2-system-components-page-in-settings/) — applies to: Windows 11 23H2 (feature under test); the article says it might also come to Windows 11 22H2. No specific build number is stated..
  - System Components (Settings page): Lists system-related apps and components, including default apps, extensions, codecs, and other programs considered system-related.. Provides a place to manage system components. The page lists system apps, including apps that cannot be uninstalled; Calculator and Notepad can be uninstalled from the new Settings page.
  - Requirement/rollout: Microsoft is testing the feature in Windows 11 23H2; the article says it might eventually be integrated into Windows 11 22H2.
  - Requirement/rollout: The article states no Copilot+ PC, Insider channel, region, or SKU requirement.


## WB.02 — Bluetooth & devices


### WB.02.01 — Bluetooth & devices > Bluetooth / Devices

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Bluetooth & devices`; `Windows 11: Start > Settings > Bluetooth & devices`; `Start > Settings > Bluetooth & devices > Devices`

ms-settings URIs: `ms-settings:bluetooth`, `ms-settings:connecteddevices`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/pair-a-bluetooth-device-in-windows-2be7b51f-6ae9-b757-a3b9-95ee40c3e242); [Official support article (T1)](https://support.microsoft.com/help/15290); [Official support article (T1)](https://support.microsoft.com/en-us/windows/hardware/add-a-hardware-device-to-a-windows-device)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.02.01.o01 | Bluetooth | Toggle | On; Off | — | Turn Bluetooth on or off in the Bluetooth & devices window. | The Windows device must support Bluetooth. If it does not have built-in Bluetooth, a USB Bluetooth adapter can be used. |
| WB.02.01.o02 | Open Bluetooth & devices | Shortcut link |  | — | Opens Bluetooth & devices. | — |
| WB.02.01.o03 | Bluetooth devices discovery | Setting; control type not stated | Advanced | — | The setting might need to be set to Advanced. | If the Bluetooth device is not displayed. |
| WB.02.01.o04 | Add device | Button |  | — | Opens the flow to add a device; follow the on-screen instructions to finish. | The device should be turned on and, if required, in pairing mode. Keep it close to the PC during setup. |
| WB.02.01.o05 | Open Devices | Link |  | — | Opens the Devices page. | — |

- Confirmations / errors: When Bluetooth is turned on, the Bluetooth device and the Windows device automatically connect whenever they are in range of each other.
- Keyboard shortcuts: Open Bluetooth & devices
- SKU / hardware / version conditions: Before starting, make sure the Windows 11 device supports Bluetooth. A USB Bluetooth adapter can add Bluetooth to a Windows device that does not have it built in.; The Windows device needs to have Bluetooth. Most Windows devices already have Bluetooth built in; a USB Bluetooth adapter can add Bluetooth to a device that does not have it.
- Windows 10 differences (legacy): The Bluetooth toggle is under Bluetooth in the Bluetooth & other devices window.; The Settings path is Start > Settings > Devices; the Bluetooth settings are under Bluetooth & other devices.; The navigation path is Start > Settings > Devices.; The page is called Bluetooth & other devices, and its button is labeled Add Bluetooth or other device.; To pair from Settings, open Bluetooth & other devices, select Add Bluetooth or other device, then select Bluetooth. Select the desired discoverable device, follow any additional instructions, and select Done.; Windows 10 Bluetooth settings path: Start > Settings > Devices > Bluetooth & other devices. Make sure Bluetooth is switched On.


### WB.02.02 — Bluetooth & devices > Printers & scanners

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Bluetooth & devices > Printers & scanners`

ms-settings URIs: `ms-settings:printers`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/hardware/printer/add-or-install-a-printer-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.02.02.o01 | Printers & scanners | Settings page |  | — | Shows the list of installed printers and the controls for adding or removing a printer. | — |
| WB.02.02.o02 | Add a printer & scanner | Section |  | — | Contains the control for adding a printer or scanner. | Shown when a printer isn't installed. |
| WB.02.02.o03 | Add device | Button | A list of available printers; select the desired printer. | — | Adds the selected printer. The printer might be added automatically without selecting this button. | Shown after selecting Add device to search for printers. |
| WB.02.02.o04 | The printer that I want isn't listed | Link or option |  | — | Opens the option to add a device manually. | Use if the printer isn't found automatically. |
| WB.02.02.o05 | Add a new device manually | Option |  | — | Opens the Add Printer window, where you can follow the instructions. | Available after selecting The printer that I want isn't listed. |
| WB.02.02.o06 | Find drivers | Button |  | — | The printer might work with reduced functionality until a full set of drivers is installed. | Might appear next to the printer after installation. |
| WB.02.02.o07 | Remove | Button |  | — | Removes the selected installed printer. | Select an installed printer first. |
| WB.02.02.o08 | Open Bluetooth & devices > Printers & scanners | Link |  | — | Opens the Printers & scanners Settings page. | — |

- Confirmations / errors: A Find drivers button might appear after installation; the printer might have reduced functionality until a full set of drivers is installed.; A printer might be added automatically without selecting Add Device.; If the printer isn't found automatically, select The printer that I want isn't listed and then Add a new device manually.; If the printer still doesn't install after trying manually, the page directs users to Fix printer connection and printing problems in Windows.
- SKU / hardware / version conditions: For a network or wireless printer, the printer must be on and connected to the same network as the Windows device; wireless printers must be on the same wireless network.; For an ARM PC, the page says to use these Windows instructions instead of the printer manufacturer's installer; follow the manufacturer's guidance if it differs. To check, go to Start > Settings > System > About and look under Device specifications for ARM based processor next to System type.
- Windows 10 differences (legacy): For manual addition, select The printer that I want isn't listed; the page does not name the Windows 11 Add a new device manually option in the Windows 10 section.; The add section is labeled Add printers & scanners, and the button is Add a printer or scanner.; The navigation path is Start > Settings > Devices > Printers & scanners.; To remove a printer, select it and then select the Remove device button.


### WB.02.03 — Bluetooth & devices > Mobile devices / Phone Link

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `On Windows: taskbar search box > type “Phone Link” > select “Phone Link” from results.`; `On Windows: Start menu > “Phone Link” (or search for it).`; `To install Phone Link: Windows Settings > search “Phone Link” > “Enable or disable Phone Link” > enable the Phone Link toggle > “Install”. On Windows 10 the toggle label is “Turn on Phone Link”; on Windows 11 it is “Phone Link”.`; `For iPhone Bluetooth cleanup: Windows Settings > “Bluetooth & Devices” > “Remove Device”.`; `Phone Link > “Settings” > “Features”.`

ms-settings URIs: `ms-settings:mobile-devices`, `ms-settings:mobile-devices-addphone`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/apps/phonelink/phone-link-requirements-and-setup); [Official support article (T1)](https://support.microsoft.com/Windows/apps/phonelink/phone-link-requirements-and-setup); [Official support article (T1)](https://support.microsoft.com/en-us/windows/apps/phonelink/manage-mobile-devices-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.02.03.o01 | Phone Link | Windows app | Open from taskbar search or the Start menu. | Available by default on most Windows 10 and Windows 11 PCs. | Opens the Phone Link experience; the page describes linking Android devices and iPhones. | Windows 10 or Windows 11; device and feature requirements differ by Android/iOS. |
| WB.02.03.o02 | Enable or disable Phone Link | Settings entry | Windows Settings search result. | — | Opens the Phone Link installation controls described on the page. | Use Settings search for “Phone Link”. |
| WB.02.03.o03 | Turn on Phone Link | Toggle | On or off. | — | Enables Phone Link and prompts for installation when enabled. | Windows 10 label; Windows 11 uses the toggle label “Phone Link”. |
| WB.02.03.o04 | Install | Button/command | Install Phone Link. | — | Adds Phone Link to the PC. | Appears in a pop-up after enabling the Phone Link toggle when the app is not installed. |
| WB.02.03.o05 | I have the Link to Windows app ready | Checkbox | Selected or not selected. | — | Confirms the Android mobile app is ready before proceeding with pairing. | Android pairing started from the PC. |
| WB.02.03.o06 | Pair with QR code | Command/button | Pair using a QR code. | — | Displays a QR code on the PC for the Link to Windows app to scan. | Android pairing started from the PC. |
| WB.02.03.o07 | Pair manually | Command/button | Pair without a QR code. | — | Prompts the user to enter a code on the Android device. | Alternative to “Pair with QR code” during Android pairing. |
| WB.02.03.o08 | Link your phone and PC | Mobile-app command | Start linking the phone and PC. | — | Starts the QR-code linking flow in Link to Windows. | Android pairing using a QR code. |
| WB.02.03.o09 | Continue | Mobile-app command | Continue after confirming the QR code on the PC is ready; also continue after Microsoft account confirmation/sign-in. | — | Advances the applicable setup flow. | Android Link to Windows setup. |
| WB.02.03.o10 | Done | Mobile-app command | Finish the phone-side setup. | — | Completes the phone-side prompts; return to the PC to use Phone Link. | Android pairing. |
| WB.02.03.o11 | Settings | Phone Link navigation item | Open Phone Link settings. | — | Provides access to Features, My Devices, and Feedback & Community as described on the page. | — |
| WB.02.03.o12 | Features | Phone Link settings tab/section | Feature controls, including Notifications, Messages, Calls, and Photos; availability can depend on the linked device. | — | Toggle features on or off to control which features and content sync. Turning a feature off stops syncing related content until re-enabled. | Some features and additional settings depend on the linked mobile device. |
| WB.02.03.o13 | Notifications | Feature toggle / permission | On or off. | — | Controls notification syncing; Android notification permission is needed for notification sending and related cross-device features. | Android permissions may be required; Android 13 requires notification permission. |
| WB.02.03.o14 | Messages | Feature toggle / permission | On or off. | — | Controls message syncing and interaction in Phone Link. | For iPhone, message access depends on the listed Bluetooth permission settings. The page warns that granted message permission can expose sensitive data such as 2-Factor Authentication SMS. |
| WB.02.03.o15 | Calls | Feature toggle | On or off. | — | Controls call-related Phone Link functionality. | Available features depend on the linked device. |
| WB.02.03.o16 | Photos | Feature toggle | On or off. | — | Controls photo syncing/access; turning it off stops photo syncing. | On Samsung devices, disabling Link to Windows under All files access removes access to photos and file sharing. |
| WB.02.03.o17 | Transferred files will save to | Setting | Change the save location for files received from the mobile device. | — | Sets the location on the linked PC where received files are saved. | May be available depending on the linked mobile device. |
| WB.02.03.o18 | Enable Audio player | Setting | Enable or disable. | — | Allows audio from the linked mobile device to be played. | May be available depending on the linked mobile device. |
| WB.02.03.o19 | Setup Instant Hotspot | Setting/command | Configure the mobile device as an instant hotspot. | — | Sets up instant hotspot functionality. | May be available depending on the linked mobile device. |
| WB.02.03.o20 | Cross-device copy and paste | Setting | Enable or disable. | — | Enables cross-device copy and paste. | Only when supported. |
| WB.02.03.o21 | Wallpaper | Setting | Enable or disable. | — | Syncs the mobile device’s wallpaper with the PC. | May be available depending on the linked mobile device. |
| WB.02.03.o22 | Sync your content over mobile data | Setting | Enable or disable. | — | Allows content syncing when Wi-Fi is unavailable or when preferred. | May be available depending on the linked mobile device. |
| WB.02.03.o23 | My Devices | Phone Link settings tab | View/manage devices; includes “Link new device”. | — | Provides the listed route for linking another iPhone and for removing a device using the Remove option. | The iPhone instructions specify this tab. |
| WB.02.03.o24 | Link new device | Command/button | Start linking a new device. | — | Generates a unique QR code for pairing a new iPhone. | Phone Link > Settings > My Devices. |
| WB.02.03.o25 | Remove | Command/button | Remove the selected device. | — | Unlinks the selected device from Phone Link. | Phone Link > Settings > My Devices; described for iPhone unlinking. |
| WB.02.03.o26 | Feedback & Community | Phone Link settings section | Feedback form. | — | Lets the user describe persistent pairing issues. | Use if pairing issues persist. |
| WB.02.03.o27 | Share System notifications settings | iPhone permission setting | Enable or disable. | — | Grants access to Notifications and Messages. | iPhone pairing; permissions are enabled on the iPhone. |
| WB.02.03.o28 | Show Notification settings | iPhone permission setting | Enable or disable. | — | Grants access to Messages. | iPhone pairing; permissions are enabled on the iPhone. |
| WB.02.03.o29 | Sync Contacts settings | iPhone permission setting | Enable or disable. | — | Grants access to Messages, Contacts, and Call History. | iPhone pairing; permissions are enabled on the iPhone. |
| WB.02.03.o30 | Bluetooth | Device setting | On or off; iPhone Bluetooth device can also be forgotten. | — | Bluetooth must be on for iPhone pairing. Forgetting the PC on the iPhone or removing the iPhone from Windows Bluetooth settings is part of the stated reset/unpair steps. | iPhone pairing requires PC Bluetooth Low Energy support and Bluetooth enabled on both devices. |
| WB.02.03.o31 | Remove Device | Windows Settings command | Remove the iPhone from Windows Bluetooth settings. | — | Removing the iPhone stops Phone Link syncing calls, messages, and notifications until it is paired again. | Windows Settings > Bluetooth & Devices. |
| WB.02.03.o32 | Forget this Device | iPhone Bluetooth command | Forget the PC. | — | Unpairs the iPhone from the PC; also used in the pairing-reset instructions. | iPhone Settings > Bluetooth > information icon next to the PC name in “My Devices”. |
| WB.02.03.o33 | Try pairing again | Command/button | Retry pairing. | — | Resumes the iPhone pairing attempt after Bluetooth has been toggled off and back on on both devices. | iPhone QR-code pairing troubleshooting. |
| WB.02.03.o34 | Link to Windows | Android Do Not Disturb permission toggle | On or off, if available. | — | Allows Phone Link to manage Do Not Disturb when applicable. | Android 15 or later; another app or mode may control Do Not Disturb. |
| WB.02.03.o35 | All files access | Android special-access permission | Manage access; Link to Windows access can be disabled. | The permission is pre-granted on Samsung mobile devices. | Disabling Link to Windows access removes Phone Link/PC access to photos and file sharing from the mobile device. | Samsung mobile device. |
| WB.02.03.o36 | Location | Android app permission | “Allow only while using the app”, “Ask every time”, or “Don’t allow”. | — | Controls Link to Windows location access. | Android Settings > Apps > Link to Windows > Permissions > Location. |
| WB.02.03.o37 | Microsoft account sign-in | Sign-in option | Sign in with an existing Microsoft account or create a new one; alternatively scan the QR code. | — | Links the mobile device to the Microsoft account used in the pairing flow. | The page says the selected/PC-setup Microsoft account is used during pairing; work/school accounts are unsupported as stated for the relevant platforms. |
| WB.02.03.o38 | Android | Device type | Select Android | — | Begins or continues Android pairing. | For pairing an Android device. |
| WB.02.03.o39 | iPhone® | Device type | Select iPhone® | — | Begins iPhone pairing. | The choice may be greyed out if PC Bluetooth is turned off or unavailable. |
| WB.02.03.o40 | Sign in with Microsoft | Sign-in option | Sign in with an existing Microsoft account or sign in/create an account | — | Signs the Link to Windows app in to a Microsoft account. | Mobile-device sign-in; an existing account may be confirmed before continuing. |
| WB.02.03.o41 | Share System notifications | iPhone setting | Enable | — | Grants access to Notifications and Messages. | Enable on the iPhone after Bluetooth pairing. |
| WB.02.03.o42 | Show Notification | iPhone setting | Enable | — | Grants access to Messages. | Enable on the iPhone after Bluetooth pairing. |
| WB.02.03.o43 | Sync Contacts | iPhone setting | Enable | — | Grants access to Messages, Contacts, and Call History. | Enable on the iPhone after Bluetooth pairing. |
| WB.02.03.o44 | How to manage your mobile device on your PC? | Section heading |  | — | Describes linking and managing an Android device from a PC. | Requires a Windows 11 PC, an Android device running Android 10 or later, and both devices connected to Wi-Fi. |
| WB.02.03.o45 | How do I link my devices? | Section heading |  | — | Provides the device-linking steps. | The PC and Android device should be nearby, turned on, and connected to Wi-Fi. |
| WB.02.03.o46 | Allow this PC to access content and capabilities from your mobile devices | Toggle | On or off | — | Allows the PC to access content and capabilities from mobile devices. | Turn it on if it is off before selecting Add device. |
| WB.02.03.o47 | Add device | Command/button |  | — | Opens a popup window displaying a QR code to scan with the mobile device. | Used to link a device. |
| WB.02.03.o48 | What devices are supported? | Section heading |  | — | Lists supported Android and Link to Windows app versions. | — |
| WB.02.03.o49 | How do I manage which features are available in Manage mobile device? | Section heading |  | — | Explains how to customize features and content shared between the PC and linked device. | A device must first be added to view and manage available features. |
| WB.02.03.o50 | Manage devices | Command/button |  | — | Opens the controls for features under the linked enabled device. | Located under Settings > Bluetooth & devices > Mobile Devices. |
| WB.02.03.o51 | Show mobile device in File Explorer | Feature toggle | On or off | — | Enables access to the mobile device’s files, photos, and media in File Explorer and interaction with them. | If disabled, files from the mobile device cannot be accessed, copied, or pasted. |
| WB.02.03.o52 | Use as a connected camera | Feature toggle | On or off | — | Makes the mobile device’s camera available as a webcam to apps. | — |
| WB.02.03.o53 | Get new photo notifications | Feature toggle | On or off | — | Provides notifications to open or edit new photos from the mobile device. | — |

- Confirmations / errors: After scanning the QR code, the user may be prompted to grant additional permissions to access mobile-device content.; Android 13 requires notification permission; the page says it can be granted during pairing or through Android Settings > Apps > Link to Windows > Notifications.; Disabling a Phone Link feature stops syncing its related content until the feature is re-enabled.; For Android 13, notification permission must be approved on the Android device; the page gives System Settings > Apps > Link to Windows > Notifications as a route to enable it.; For iPhone pairing, the iPhone choice may be greyed out if PC Bluetooth is off or unavailable.; For iPhone, if “iPhone” is greyed out under “Pick your phone”, the PC’s Bluetooth may be off or unavailable.; If an iPhone is removed from Windows Bluetooth settings, Phone Link will no longer sync calls, messages, and notifications; forget the PC on the iPhone and initiate pairing again in Phone Link.; If iPhone QR pairing fails, ensure Bluetooth is on and the PC and iPhone are discoverable to each other; the page suggests removing the Bluetooth pairings, toggling Bluetooth off for 10 seconds on both devices, turning it back on, and selecting “Try pairing again”.; If iPhone QR pairing fails, the page recommends checking Bluetooth and discoverability on both devices; if needed, remove each device from the other’s Bluetooth settings, turn Bluetooth off on both for 10 seconds, turn it back on, and select “Try pairing again.”; If iPhone pairing problems persist, use the form under Phone Link Settings > Feedback & Community or search for Feedback Hub.; On Android 15 or later, Do Not Disturb control may not work as expected if another app or mode manages it or Link to Windows Do Not Disturb access is disabled.; On Android 15 or later, Do Not Disturb may not respond as expected if another app or mode manages it, or if Link to Windows Do Not Disturb access is disabled.; Phone Link does not currently support multiple Android profiles or work/school accounts; the iOS requirements section also says Phone Link does not support work/school accounts.; The page does not identify a Windows Settings page or path named “Mobile devices”; it describes the Phone Link app and related settings and Bluetooth paths.; The page says Phone Link does not currently support multiple Android profiles or work/school accounts; for iPhone it says Phone Link does not support work/school accounts.; The page states that granting Microsoft account linking permission transfers Microsoft account credentials to the phone; contact syncing with the Microsoft account can make phone-synced contacts available in other Microsoft apps and services.; The page warns that disabling a feature affects what and how content can be accessed. It gives disabling Show mobile device in File Explorer as an example: files cannot be accessed, copied, or pasted.
- Restart / sign-out: For a fresh iPhone pairing state, remove the iPhone from Windows Bluetooth settings and the PC from iPhone Bluetooth settings, turn Bluetooth off on both devices, wait 10 seconds, turn Bluetooth back on, and select “Try pairing again.”; For iPhone pairing reset: remove the iPhone from Windows Bluetooth settings and forget the PC on the iPhone, then toggle Bluetooth off on both devices for 10 seconds, turn it back on, and retry pairing.; For iPhone re-pairing after removal from Windows Bluetooth settings, first forget the PC in the iPhone Bluetooth settings, then initiate pairing again in Phone Link.; The page describes unlinking/removing a device: Phone Link > Settings > My Devices > Remove; for iPhone, also forget the PC in iPhone Bluetooth settings.; The page does not describe a sign-out command for Phone Link.; The page says devices can be unlinked at any time using the steps on its “Unlink a device” page.
- SKU / hardware / version conditions: Additional Phone Link settings and supported features vary depending on the linked mobile device.; Android Phone Link requirements: Windows 10 with the October 2022 update or later, or Windows 11; Android 10 or later; phone and PC on the same Wi-Fi network. The page recommends the latest Windows version.; Android device must run Android 10 or later.; Android linking is described for select Android, Samsung, HONOR, OPPO, ASUS, and vivo devices with Link to Windows preinstalled or available from Google Play or Samsung Galaxy Store.; Android linking is described for select Android, Samsung, HONOR, OPPO, ASUS, and vivo devices. Link to Windows may be preinstalled or downloadable from Google Play or Samsung Galaxy Store.; Android: Windows 10 with the October 2022 update or later, or Windows 11; Android 10 or later; PC and phone on the same Wi-Fi network.; For optional iPhone Link to Windows app installation, the page recommends app version 1.23031.2 or later; downloading the app is not required.; For the optional iPhone Link to Windows app, the page recommends version 1.23031.2 or later.; PC must run Windows 11.; Phone and PC must both be connected to a Wi-Fi network; they should also be nearby and turned on.; Supported Link to Windows app version: 1.23112.189 or later.; The page notes that select supported Samsung tablets have separate onboarding steps.; The page recommends the latest Windows version for the best experience.; The page refers separately to onboarding steps for select supported Samsung tablets.; iPhone Phone Link requirements: Windows 10 with the May 2019 Update or later, or Windows 11; PC with Bluetooth Low Energy (BLE) support; iOS 16 or later.; iPhone pairing requires Bluetooth enabled on both devices. The page says the PC and iPhone should be nearby.; iPhone: Windows 10 with the May 2019 Update or later, or Windows 11; PC with Bluetooth Low Energy (BLE) support; iOS 16 or later.
- Windows 10 differences (legacy): Android Phone Link requirements specify Windows 10 with the October 2022 update or later; iPhone requirements specify Windows 10 with the May 2019 Update or later.; Android Phone Link requires Windows 10 with the October 2022 update or later; iPhone Phone Link requires Windows 10 with the May 2019 Update or later.; Phone Link installation uses the toggle label “Turn on Phone Link” on Windows 10; the Windows 11 instructions label the toggle “Phone Link.” Both instructions use Settings search for Phone Link, select “Enable or disable Phone Link,” and then select “Install” in the pop-up.; The installation toggle is labeled “Turn on Phone Link” on Windows 10; the Windows 11 label is “Phone Link”.; The page does not state other Windows 10 versus Windows 11 differences for the described Phone Link controls.; The page gives the same general Phone Link installation Settings search flow for Windows 10 and Windows 11, but does not state a separate Windows 10/11 “Mobile devices” Settings page.; The page specifies Windows 11 as a requirement and does not describe a Windows 10 version or differences.


### WB.02.04 — Bluetooth & devices > Cameras

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Bluetooth & devices > Cameras`; `Settings > Bluetooth & devices > Cameras > [select camera] > Edit > Advanced camera options`

ms-settings URIs: `ms-settings:camera`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/hardware/camera/manage-cameras-with-camera-settings-in-windows-11)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.02.04.o01 | Search for cameras | Button |  | — | Searches the local network for ONVIF-conformant network/IP cameras. | Used to connect a network camera. |
| WB.02.04.o02 | Connected cameras | List and camera subpages |  | — | Lists cameras currently connected to the system. Selecting a camera opens a subpage with a preview, camera management, and available settings. | Some specialty cameras may not appear, including IR cameras used for Windows Hello Face, DirectShow cameras, and proprietary cameras not managed by Windows. |
| WB.02.04.o03 | user name and password | Credentials prompt |  | — | Requests the camera's credentials. | May appear when connecting a network camera. |
| WB.02.04.o04 | Remove | Button |  | — | Removes a selected connected network camera. | Available for connected network cameras. |
| WB.02.04.o05 | Disabled cameras | Section |  | — | Lists cameras disabled through the camera settings page. | The section appears when a camera is disabled; it is absent when no cameras are disabled. |
| WB.02.04.o06 | Enable | Button |  | — | Re-enables the selected disabled camera. | Administrator access is required. |
| WB.02.04.o07 | Disable | Button |  | — | Disables the selected camera after a confirmation prompt. | Administrator access is required. Some systems may require a reboot; cameras sharing processing hardware may be disabled as a group. |
| WB.02.04.o08 | Troubleshoot | Button |  | — | Launches the Get Help utility's interactive camera troubleshooting experience. | Available on the selected camera's subpage. |
| WB.02.04.o09 | Basic settings | Section with sliders and/or toggles | Zoom; Pan; Tilt; Brightness; Contrast; Sharpness; Saturation; Video HDR | — | Provides basic imaging controls supported by the camera. Changes are applied to the live stream and saved as defaults for future camera use. | Controls are optional; some or all may be unavailable, and the section may not appear. |
| WB.02.04.o10 | Windows Studio Effects | Section with camera controls | Automatic framing; Portrait light; Eye contact; Background effects; Creative filters; and more | — | Provides supported camera effects. | May be available on Copilot+ PCs. Some cameras provide these controls independently, in a section labeled Camera Effects. |
| WB.02.04.o11 | Camera Effects | Section with camera controls | Automatic framing; Portrait light; Eye contact; Background effects; Creative filters; and more | — | Provides supported camera effects. | This label may be used instead of Windows Studio Effects when a camera implements the controls directly. |
| WB.02.04.o12 | Video rotation | Dropdown | No rotation; Right 90°; Left 90°; 180° | — | Sets the camera video rotation. A change may not apply until the camera is next used if another application is using it. Non-No rotation settings remove compressed data types and output uncompressed data types. | Available only for external cameras. |
| WB.02.04.o13 | Reset settings | Button |  | — | Resets camera settings to factory defaults and restarts the camera. Erases changes to Basic settings, Windows Studio Effects or Camera Effects, and Video rotation; also resets defaults configured by a camera manufacturer's companion app. | Available on a selected camera's subpage. |
| WB.02.04.o14 | Related settings | Section containing a link |  | — | A link opens the camera manufacturer's companion app or its Microsoft Store page. | Available for cameras whose manufacturers provide a companion app. |
| WB.02.04.o15 | Advanced camera options | Submenu | Multi-app camera; Basic camera | — | Contains controls for camera modes. | Available on Windows 11, version 24H2 and later; options might be managed by an organization. |
| WB.02.04.o16 | Edit | Button |  | — | Opens the Advanced camera options submenu. | Located under Advanced camera options after selecting a camera. |
| WB.02.04.o17 | Multi-app camera | Mode toggle and resolution/frame-rate controls | Choose a resolution and frame rate, or let Windows determine them. | When Windows determines them, it attempts to select the highest available camera resolution up to 1440p that can stream at 30 fps. | Allows multiple applications to access the camera stream simultaneously. | If Windows Hello Face Recognition tries to use a camera in multi-app managed mode while another app is using it, Hello cannot access the camera and falls back to PIN login. The listed resolutions and frame rates may not include all supported options. |
| WB.02.04.o18 | Basic camera | Mode toggle and resolution/frame-rate controls | Choose a resolution and frame rate, or let Windows determine them. | When Windows determines them, it attempts to select the highest available camera resolution up to 1440p that can stream at 30 fps. | Turns off hardware acceleration and, where possible, camera manufacturer plug-ins. Enabling this mode also enables multi-app camera mode. | A troubleshooting option. Windows Studio Effects will not work while it is on. Windows Hello Face Recognition bypasses this mode. |

- Confirmations / errors: A camera may remain greyed out in Connected cameras with a message indicating that the PC must be rebooted before the camera is disabled.; An application monitoring a control may rewrite it after an external change, which can cause a mismatch between the preview and camera settings.; Cameras may not appear if they are missing a driver, are nonfunctional, or were disabled through another utility such as Device Manager.; Changing Video rotation away from No rotation removes compressed data types (for example, MJPEG and H.264) and may cause legacy applications or cameras that depend on compressed formats to malfunction.; Disabling a camera triggers a prompt to confirm before disabling it.; Disabling one camera can disable multiple cameras when they share camera-processing hardware; the confirmation prompt indicates this when applicable.; When another application is using the camera, an information banner indicates that changes made in Camera settings or in the application will affect both the application and the preview.
- Restart / sign-out: In some circumstances, disabling a camera requires a reboot of the system.; Reset settings restarts the camera.
- Keyboard shortcuts: WIN + F — opens Feedback Hub.
- SKU / hardware / version conditions: Administrator access is required to enable or disable cameras.; Advanced camera options are available on Windows 11, version 24H2 and later, and might be managed by an organization.; Camera settings vary by camera support; specialty cameras may not appear on the page.; Some systems have cameras that share camera-processing hardware.; The Camera settings page was introduced in Windows 11 and is not available on older versions of Windows.; Video rotation is available only for external cameras.; Windows Studio Effects may be available on Copilot+ PCs.
- Windows 10 differences (legacy): The page states that Camera settings was introduced in Windows 11 and is not available on older versions of Windows.


### WB.02.05 — Bluetooth & devices > Mouse

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Bluetooth & devices > Mouse`; `Bluetooth & devices > Mouse`; `Bluetooth & devices > Mouse > Related settings > Additional mouse settings`

ms-settings URIs: `ms-settings:mousetouchpad`

Sources: [Official support article (T1)](https://support.microsoft.com/en-US/Windows/Hardware/input-devices/change-mouse-settings)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.02.05.o01 | Primary mouse button | Drop-down menu | Left; Right | — | Selects the primary mouse button. | — |
| WB.02.05.o02 | Mouse pointer speed | Slider |  | — | Sets the mouse pointer speed. | — |
| WB.02.05.o03 | Enhance pointer precision | Toggle | On | — | Makes the pointer more accurate when the mouse is moving slowly. | — |
| WB.02.05.o04 | Additional mouse settings | Link |  | — | Opens the Mouse Properties window. | Located under Related settings. |
| WB.02.05.o05 | Roll the mouse wheel to scroll | Drop-down menu | Multiple lines at a time; One screen at a time | — | Chooses how the mouse wheel scrolls. | Located under Scrolling. |
| WB.02.05.o06 | Lines to scroll at a time | Slider |  | — | Sets the number of lines the screen scrolls for each notch of mouse wheel movement. | Located under Scrolling. |
| WB.02.05.o07 | Scroll inactive windows when hovering over them | Toggle | On | — | Allows scrolling through a window that isn't active by hovering over it. | Located under Scrolling. |
| WB.02.05.o08 | Scrolling direction | Drop-down menu | Down motion scrolls down; Down motion scrolls up | — | Sets the scrolling direction. | Located under Scrolling. |

- Keyboard shortcuts: CTRL
- Windows 10 differences (legacy): Enhance pointer precision is accessed through Related settings > Additional mouse options and the Pointer Options tab, rather than being a toggle on the Mouse page.; The Windows 10 section does not mention a Scrolling direction setting.; The navigation path is Start > Settings > Devices > Mouse.; The pointer-speed control is labeled Cursor speed, rather than Mouse pointer speed.; The primary-button control is labeled Select your primary button, rather than Primary mouse button.; The scrolling-lines slider is labeled Choose how many lines to scroll each time, rather than Lines to scroll at a time.


### WB.02.06 — Bluetooth & devices > Touchpad

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Bluetooth & devices > Touchpad`; `Settings > Bluetooth & devices > Touchpad`; `Open Touchpad settings`

ms-settings URIs: `ms-settings:devices-touchpad`

Sources: [Official support article (T1)](https://support.microsoft.com/help/17206/change-mouse-settings); [Official support article (T1)](https://support.microsoft.com/en-au/windows/touch-gestures-for-windows-a9d28305-4818-a5df-4e2b-e5590f850741); [Official support article (T1)](https://support.microsoft.com/en-us/surface/cover-keyboard/surface-touchpad-use-and-settings)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.02.06.o01 | Three-finger gestures | Expandable section with drop-down menus |  | — | Customize the action performed by three-finger gestures. | — |
| WB.02.06.o02 | Four-finger gestures | Expandable section with drop-down menus |  | — | Customize the action performed by four-finger gestures. | — |
| WB.02.06.o03 | Open Touchpad settings | Link |  | — | Opens Touchpad settings. | — |
| WB.02.06.o04 | Select an item | Touchpad gesture | Tap touchpad. | — | Selects an item. | — |
| WB.02.06.o05 | Scroll | Touchpad gesture | Place two fingers on the touchpad and slide horizontally or vertically. | — | Scrolls horizontally or vertically. | — |
| WB.02.06.o06 | Zoom in or out | Touchpad gesture | Place two fingers on the touchpad and pinch in or stretch out. | — | Zooms in or out. | — |
| WB.02.06.o07 | Show more commands (like right-click) | Touchpad gesture | Tap the touchpad with two fingers or press down in the lower-right corner. | — | Shows more commands. | — |
| WB.02.06.o08 | Show all open windows | Touchpad gesture | Swipe with three fingers up on the touchpad. | — | Shows all open windows. | Some touchpad gestures only work with precision touchpads. |
| WB.02.06.o09 | Show the desktop | Touchpad gesture | Swipe with three fingers down on the touchpad. | — | Shows the desktop. | Some touchpad gestures only work with precision touchpads. |
| WB.02.06.o10 | Switch between open apps or windows | Touchpad gesture | Swipe with three fingers left or right on the touchpad. | — | Switches between open apps or windows. | Some touchpad gestures only work with precision touchpads. |
| WB.02.06.o11 | Switch desktops | Touchpad gesture | Swipe with four fingers to the left or right on the touchpad. | — | Switches desktops. | Some touchpad gestures only work with precision touchpads. |
| WB.02.06.o12 | turn the touchpad on and off | Not specified | On or off | — | Turn the touchpad on or off. | The page says to attach the Surface to its keyboard or Type Cover before changing touchpad settings. |
| WB.02.06.o13 | adjust cursor speed | Not specified | Not specified | — | Adjust cursor speed. | The page says to attach the Surface to its keyboard or Type Cover before changing touchpad settings. |
| WB.02.06.o14 | reverse scrolling direction | Not specified | Not specified | — | Reverse scrolling direction. | The page says to attach the Surface to its keyboard or Type Cover before changing touchpad settings. |
| WB.02.06.o15 | change three-finger and four-finger gestures | Not specified | Not specified | — | Change three-finger and four-finger gestures. | The page says to attach the Surface to its keyboard or Type Cover before changing touchpad settings. |
| WB.02.06.o16 | more | Not specified | Not specified | — | The page says other touchpad settings can also be adjusted, but does not specify them. | The page says to attach the Surface to its keyboard or Type Cover before changing touchpad settings. |

- SKU / hardware / version conditions: Some of these touchpad gestures only work with precision touchpads.; Some touchpad gestures only work with precision touchpads.; The page says Surface app settings and customization options depend on the specific type of Surface device.; The settings instructions say to attach your Surface to its keyboard or Type Cover.
- Windows 10 differences (legacy): The Windows 10 content explains that a precision touchpad can be identified by the message “Your PC has a precision touchpad.” at the top of the Touchpad window.; The Windows 10 touchpad gesture list also states: tap three fingers to open Cortana; tap four fingers to open action center.; The Windows 10 touchpad settings path is Settings > Devices > Touchpad.; The navigation path is Start > Settings > Devices > Touchpad.; The page says a precision touchpad is identified by the message “Your PC has a precision touchpad.”; Windows 10 describes the drop-down menus in the Three-finger gestures and Four-finger gestures sections without saying to expand those sections.; Windows 10 path: Start > Settings > Devices > Touchpad.


### WB.02.07 — Bluetooth & devices > Pen & Windows Ink

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Bluetooth & devices > Pen & Windows Ink`; `Windows 11: Start > Settings > Bluetooth & devices > Pen & Windows Ink`; `Settings > Bluetooth & devices > Pen & Windows Ink`

ms-settings URIs: `ms-settings:pen`

Sources: [Official support article (T1)](https://support.microsoft.com/en-US/windows/hardware/input-devices/use-a-pen-with-windows); [Official support article (T1)](https://support.microsoft.com/Windows/hardware/input-devices/use-a-pen-with-windows); [Official support article (T1)](https://support.microsoft.com/ja-jp/help/4023514/surface-change-pen-settings) (dated 2026-06-30)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.02.07.o01 | Choose which hand you write with |  |  | — | The page says you can choose which hand you write with. | — |
| WB.02.07.o02 | what your PC does when you tap, double-tap, or hold down the pen's shortcut button |  |  | — | The page says you can customize what your PC does for these pen actions. | The pen must have a shortcut button for the shortcut-button actions. |
| WB.02.07.o03 | Open Pen & Windows Ink Settings | Link | Not stated | — | Opens Pen & Windows Ink settings. | Not stated |
| WB.02.07.o04 | 書き込む手 | 設定できる項目として説明されていますが、個別のUIラベルやコントロールの種類は記載されていません。 |  | — | 書き込む手を選択できます。 | — |
| WB.02.07.o05 | PC の動作 | ペンのショートカット ボタンの操作に関する設定として説明されていますが、個別のUIラベルやコントロールの種類は記載されていません。 |  | — | ショートカット ボタンをタップ、ダブルタップ、または長押ししたときの PC の動作を選択できます。 | — |

- Confirmations / errors: ページには、この設定ページにある個々のトグル、ドロップダウン、ボタン、リンクの一覧や正確なUIラベルは記載されていません。; 既定値は記載されていません。
- SKU / hardware / version conditions: The described shortcut-button behavior applies to a pen with a shortcut button.; The shortcut-button settings apply when the pen has a shortcut button.
- Windows 10 differences (legacy): The Windows 10 path is Start > Settings > Devices > Pen & Windows Ink.; Windows 10 settings path: Start > Settings > Devices > Pen & Windows Ink. The page does not state Windows 10-specific setting names, choices, or defaults.; Windows 10 の設定へのパスは「Settings > Devices > Pen & Windows Ink」と記載されています。


### WB.02.09 — Bluetooth & devices > AutoPlay

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Settings > Bluetooth & devices > AutoPlay`; `Bluetooth & devices > AutoPlay`; `Windows 11: Settings > Bluetooth & devices > AutoPlay`

ms-settings URIs: `ms-settings:autoplay`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/windows-backup-settings-catalog-deebcba2-5bc0-4e63-279a-329926955708?WT.mc_id=AZ-MVP-5003535); [Official documentation (T1)](https://learn.microsoft.com/en-us/windows/apps/develop/settings/settings-common) (dated 2024-03-04); [Official support article (T1)](https://support.microsoft.com/en-us/windows/experience/backup-recovery/windows-backup-settings-catalog)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.02.09.o01 | Use AutoPlay for all media and devices | Toggle |  | — |  | — |
| WB.02.09.o02 | Choose AutoPlay removable drive default | Default-selection control |  | — |  | — |
| WB.02.09.o03 | Choose AutoPlay memory card default | Default-selection control |  | — |  | — |
| WB.02.09.o04 | Toggle the use of autoplay for all media and devices. | Registry value DisableAutoplay (REG_DWORD) under HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\AutoplayHandlers | 0/1 | — | Controls the use of AutoPlay for all media and devices. | — |
| WB.02.09.o05 | User selected default autoplay behavior for removable drive. | Registry value UserChosenExecuteHandlers\CameraAlternate\ShowPicturesOnArrival (REG_SZ) under HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\AutoplayHandlers | Import Photos and Videos (Photos); Play media on arrival.; Open folder.; Prompt each time.; Import Photos and Videos (OneDrive).; Take no action. | — | Stores the user's selected default AutoPlay behavior for a removable drive. | — |
| WB.02.09.o06 | User selected default autoplay behavior for memory card. | Registry value UserChosenExecuteHandlers\ StorageOnArrival (REG_SZ) under HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\AutoplayHandlers | Take no action.; Open folder.; Configure Storage Settings (Settings).; Prompt each time. | — | Stores the user's selected default AutoPlay behavior for a memory card. | — |
| WB.02.09.o07 | System default autoplay behavior for removable drive. | Registry value EventHandlersDefaultSelection\CameraAlternate\ShowPicturesOnArrival (REG_SZ) under HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\AutoplayHandlers | Import Photos and Videos (Photos); Play media on arrival.; Open folder.; Prompt each time.; Import Photos and Videos (OneDrive).; Take no action. | — | Stores the system default AutoPlay behavior for a removable drive. | — |
| WB.02.09.o08 | System default autoplay behavior for memory card. | Registry value EventHandlersDefaultSelection\StorageOnArrival (REG_SZ) under HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\AutoplayHandlers | Take no action.; Open folder.; Configure Storage Settings (Settings).; Prompt each time. | — | Stores the system default AutoPlay behavior for a memory card. | — |

- Windows 10 differences (legacy): The Windows 10 section lists the same AutoPlay settings under Devices > AutoPlay; it does not state different choices or defaults.; The Windows 10 section lists the same AutoPlay settings under Devices > Autoplay.


### WB.02.10 — Bluetooth & devices > USB

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Search > usb > USB settings`; `Windows 11: Select Search, search for “usb,” then select “USB settings” from the results. The page does not state the path “Bluetooth & devices > USB.”`; `Windows 11: Select Search, enter “usb,” then select “USB settings” from the list of results. The page does not state the navigation path “Bluetooth & devices > USB.”`

ms-settings URIs: `ms-settings:usb`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/hardware/usb/fix-usb-c-problems-in-windows.); [Official support article (T1)](https://support.microsoft.com/topic/f4e0e529-74f5-cdae-3194-43743f30eed2); [Official support article (T1)](https://support.microsoft.com/en-gb/topic/usb-port-may-stop-working-after-you-remove-or-insert-a-usb-device-1eaf82a6-04b1-2604-f096-2345d9c215ef)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.02.10.o01 | Open USB settings | Link |  | — | Opens USB settings. | Windows 11 |
| WB.02.10.o02 | Connection notifications | Toggle | On or off | — | Changes whether you see notifications about USB issues. | Windows 11 |
| WB.02.10.o03 | USB settings | Search result/navigation item |  | — | Selecting it opens USB settings. | — |

- Windows 10 differences (legacy): The Windows 10 setting is labeled “Notify me if there are issues connecting to USB devices”; select or clear its checkbox.; The Windows 10 setting label is “Notify me if there are issues connecting to USB devices”; select or clear its checkbox. This differs from the Windows 11 “Connection notifications” toggle.; The page gives the Windows 10 path as Start > Settings > Devices > USB.; Windows 10 checkbox label: “Notify me if there are issues connecting to USB devices.” Select or clear the checkbox to control notifications.; Windows 10 instructions say to select Start > Settings > Devices > USB.; Windows 10 path: Start > Settings > Devices > USB.


### WB.02.11 — Bluetooth & devices > Shared audio

Status: PARTIAL. Only independent or preview-era evidence; no official Microsoft documentation found for this page.


Preview/independent evidence (26H2-era):

- [Independent (T4)](https://windowsforum.com/news/windows-11-shared-audio-preview-stream-to-two-le-audio-headsets.425572/) — applies to: The page lists KB5094126, build 26200.8655, Windows 11 25H2, released June 9, 2026. It also says the update brings Shared Audio to compatible Windows 11 24H2 and 25H2 systems. The preview is described as starting on October 31, 2025..
  - Use LE Audio when available (The page says the Windows Settings app can expose this option for a paired headset.): Not stated. — default: Not stated.. The page presents the option as an indication that the paired headset supports LE Audio.
  - Shared Audio (Quick Settings tile, not described as a control on the Settings page.): Select compatible accessories; the page describes sharing audio with two devices. — default: Not stated.. Sends synchronized system audio to two supported Bluetooth LE Audio devices.
  - Requirement/rollout: The preview is described as limited to select Windows 11 Copilot+ PCs.
  - Requirement/rollout: Preview prerequisites listed include the right Insider channel, current OS and driver updates, and accessories with LE Audio enabled.
  - Requirement/rollout: The listed supported PC group includes select Surface Laptop and Surface Pro models with Qualcomm Snapdragon X, and Samsung Galaxy Book4 Edge. Certain Galaxy Book5 models with Intel Core Ultra Series 200 are described as coming soon.
  - Requirement/rollout: The page says broader Windows 11 support is planned, but gives no general release date.
  - Requirement/rollout: The page does not state a region or Windows SKU requirement.
- [Official Insider/Experience blog (T1, preview-scoped)](https://blogs.windows.com/windows-insider/2025/10/31/extending-bluetooth-le-audio-on-windows-11-with-shared-audio-preview/) — applies to: Windows 11 Insider Preview Build 26220.7051 for Dev and Beta channels begins the gradual rollout. The March 5 update says the improved preview is available with Windows Insider Build 26300.7939 or later on Dev and Beta channels..
  - Shared audio (preview) (Quick Settings tile): Not stated — default: Not stated. Opens the shared-audio experience; after two compatible accessories are paired and connected, use Share to start sharing.
  - Share (Button): Not stated — default: Not stated. Starts sharing audio with two connected compatible Bluetooth LE Audio accessories.
  - Stop sharing (Button): Not stated — default: Not stated. Ends an active sharing session.
  - Toggle to get the latest updates as they are available (Toggle in Windows Update): ON or off — default: Not stated. Turning it ON opts into receiving gradually rolled-out features as they become available. With it off, features are gradually rolled out over time once ready.
  - Requirement/rollout: The page does not mention a Settings page named “Bluetooth & devices > Shared audio.” It describes the “Shared audio (preview)” tile in Quick Settings instead.
  - Requirement/rollout: The preview is limited to select Windows 11 Copilot+ PCs with compatible Bluetooth and audio driver updates; the page lists supported models and says additional PCs may be supported later.
  - Requirement/rollout: Enroll a compatible PC in the Windows Insider Dev or Beta channel and install the latest OS and driver updates through Windows Update.
  - Requirement/rollout: For PCs listed as coming soon, driver updates were expected through Windows Update; the tile appears when the necessary driver updates are installed.
  - Requirement/rollout: The page does not state a region or Windows SKU requirement.


### WB.02.12 — Bluetooth & devices > Touch

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Bluetooth & devices > Touch`

Sources: [Official support article (T1)](https://support.microsoft.com/help/17206/change-mouse-settings)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.02.12.o01 | Open Touch settings | Link |  | — | Opens Touch settings. | — |
| WB.02.12.o02 | Three and four-finger touch gestures | Toggle | On, Off | — | Enables or disables three- and four-finger touch gestures. When enabled, three- and four-finger interactions in some apps might not work; turn this setting off to continue using those interactions in those apps. | — |
| WB.02.12.o03 | Touch screen edge gestures | Expandable section |  | — | Expands to show the screen-edge gesture settings. | — |
| WB.02.12.o04 | Swipe from the left edge of the screen | Toggle | On, Off | — | Enables or disables swiping in from the left edge of the screen. | Located under “Touch screen edge gestures”. |
| WB.02.12.o05 | Swipe from the right edge of the screen | Toggle | On, Off | — | Enables or disables swiping in from the right edge of the screen. | Located under “Touch screen edge gestures”. |

- Windows 10 differences (legacy): The Windows 10 section does not state a Settings path or configuration options for the Touch settings page.


## WB.03 — Network & internet


### WB.03.01 — Network & internet > Wi-Fi

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Settings > Network & internet > Wi-Fi`; `Settings > Network & internet > Wi-Fi > Manage known networks`; `Settings > Network & internet > Properties > Wi-Fi network password`

ms-settings URIs: `ms-settings:network-wifi`, `ms-settings:network-wifisettings`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/experience/connectivity-networking/connect-to-a-wi-fi-network-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.03.01.o01 | Random hardware addresses | Toggle |  | — | When turned on for all networks, random hardware addresses are used while the PC scans for networks and connects to any network. | The Wi-Fi hardware must support random hardware addresses. |
| WB.03.01.o02 | Manage known networks | Sub-page |  | — | Lists saved networks; selecting a network provides access to its network-specific options, including Random hardware addresses and Wi-Fi network password. | — |
| WB.03.01.o03 | Properties | Page or navigation item |  | — | The page directs users to Properties to find the Wi-Fi network password. | — |
| WB.03.01.o04 | Wi-Fi network password | Setting or page |  | — | Displays the Wi-Fi network password option. | The page says this is available under Network & internet > Properties. |
| WB.03.01.o05 | Show | Button |  | — | Reveals the Wi-Fi network password. | Can be selected next to Wi-Fi network password; for saved networks, select a network in Manage known networks first. |

- SKU / hardware / version conditions: Random hardware addresses are available only if the Wi-Fi hardware supports them.
- Windows 10 differences (legacy): For a specific network, the page says to go to Network & Internet > Wi-Fi > Manage known networks, choose a network, select Properties, and turn on Use random hardware addresses for this network.; The all-networks toggle is labeled Use random hardware addresses, and the page gives its path as Network & Internet > Wi-Fi.; To view the password, Windows 10 uses Network & Internet > Status > Network and Sharing Center > the Wi-Fi network name > Wireless Properties > Security > Show characters.


### WB.03.02 — Network & internet > Ethernet

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Network & internet > Ethernet`

ms-settings URIs: `ms-settings:network-ethernet`

Sources: [Official support article (T1)](https://support.microsoft.com/help/15291)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.03.02.o01 | Properties | Link or page |  | — | Quickly shows details about the connected network, including the IP address listed next to IPv4 address. | For an Ethernet connection, select Ethernet first. |
| WB.03.02.o02 | Network profile type | Setting | Public network (Recommended); Private network | When first connecting to a network in Windows 11, it is set as Public network by default. | Public network hides the PC from other devices and does not allow file and printer sharing. Private network makes the PC discoverable and allows file and printer sharing. | Choose the Ethernet connection, then select an option under Network profile type. Private is intended for networks whose people and devices you know and trust. |
| WB.03.02.o03 | IP assignment | Setting with Edit button | Automatic (DHCP); Manual | — | Edit opens Edit IP settings. Automatic (DHCP) sets IP address and DNS server settings automatically through the router or other access point. Manual allows those settings to be entered manually. | For an Ethernet network, select Ethernet and then select Edit next to IP assignment. |
| WB.03.02.o04 | Edit IP settings | Settings page or dialog | Automatic (DHCP); Manual | — | Provides the IP assignment choices and, when Manual is selected, fields for configuring IP and DNS settings. | Reached by selecting Edit next to IP assignment. |
| WB.03.02.o05 | IPv4 | Toggle | On; Off | — | When enabled in Manual mode, allows manual IPv4 address, gateway, and DNS settings. | Under Edit IP settings, choose Manual, then turn on IPv4. |
| WB.03.02.o06 | IP address | Text field |  | — | Enter the IPv4 or IPv6 address when configuring that protocol manually. | Manual mode; the applicable IPv4 or IPv6 toggle must be on. |
| WB.03.02.o07 | Subnet mask | Text field |  | — | Enter the IPv4 subnet mask. | Manual IPv4 configuration. |
| WB.03.02.o08 | Gateway | Text field |  | — | Enter the gateway address. | Manual IPv4 or IPv6 configuration. |
| WB.03.02.o09 | Preferred DNS | Text field |  | — | Enter the primary DNS server address. | Manual IPv4 or IPv6 configuration. |
| WB.03.02.o10 | Alternate DNS | Text field |  | — | Enter the secondary DNS server address. | Manual IPv4 or IPv6 configuration. |
| WB.03.02.o11 | DNS over HTTPS | Dropdown or setting | Off; On (automatic template); On (manual template) | — | Off sends DNS queries unencrypted. On (automatic template) encrypts DNS queries over HTTPS using default settings or by trying to discover them automatically. On (manual template) encrypts DNS queries over HTTPS using the settings entered in DNS over HTTPS template. | Appears for manually specified DNS servers in IPv4 or IPv6 settings. Not available in Windows 10. |
| WB.03.02.o12 | DNS over HTTPS template | Text field |  | — | Specifies the settings used for DNS over HTTPS. | Used with On (manual template). |
| WB.03.02.o13 | Fallback to plaintext | Toggle | On; Off | — | When on, a DNS query is sent unencrypted if it cannot be sent over HTTPS. When off, the query is not sent if it cannot be sent over HTTPS. | Available when using DNS over HTTPS with an automatic or manual template. |
| WB.03.02.o14 | IPv6 | Toggle | On; Off | — | When enabled in Manual mode, allows manual IPv6 address, gateway, and DNS settings. | Under Edit network IP settings or Edit IP settings, choose Manual, then turn on IPv6. |
| WB.03.02.o15 | Subnet prefix length | Text field |  | — | Enter the IPv6 subnet prefix length. | Manual IPv6 configuration. |
| WB.03.02.o16 | Save | Button |  | — | Saves the manually entered IP and DNS settings. | Select when finished configuring manual IPv4 or IPv6 settings. |
| WB.03.02.o17 | IPv4 address | Displayed network detail |  | — | Shows the connected Ethernet network's IP address. | Find it in the network details after selecting Ethernet. |

- Windows 10 differences (legacy): DNS over HTTPS setting is not available in Windows 10.


### WB.03.03 — Network & internet > VPN

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Network & internet > VPN`; `Network & internet > VPN > Add VPN`

ms-settings URIs: `ms-settings:network-vpn`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/experience/connectivity-networking/connect-to-a-vpn-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.03.03.o01 | Open VPN settings | Link |  | — | Opens VPN settings. | — |
| WB.03.03.o02 | Add VPN | Button |  | — | Opens the form for adding a VPN connection. | — |
| WB.03.03.o03 | VPN provider | Dropdown | Windows (built-in) | — | Choose the VPN provider. | Under Add a VPN connection. |
| WB.03.03.o04 | Connection name | Text box |  | — | Enter a recognizable name for the VPN connection. | Under Add a VPN connection. |
| WB.03.03.o05 | Server name or address | Text box |  | — | Enter the VPN server address. | Under Add a VPN connection. |
| WB.03.03.o06 | VPN type | Dropdown | The type of VPN connection used by the company or VPN service. | — | Choose the type of VPN connection to create. | The user needs to know which type the company or VPN service uses. |
| WB.03.03.o07 | Type of sign-in info | Dropdown | Username and password, one-time password, certificate, or smart card. | — | Choose the credentials to use. | Under Add a VPN connection. |
| WB.03.03.o08 | Username | Text box |  | — | Enter the username. | Enter credentials if required. |
| WB.03.03.o09 | Password | Text box |  | — | Enter the password. | Enter credentials if required. |
| WB.03.03.o10 | Save | Button |  | — | Saves the VPN profile. | After entering the VPN connection details. |
| WB.03.03.o11 | Advanced options | Button |  | — | Provides access to edit VPN connection information and additional settings, such as proxy settings. | Select a VPN connection on the VPN settings page. |
| WB.03.03.o12 | Edit | Button |  | — | Edits the selected details. | Select Advanced options, then select Edit next to the details to update. |
| WB.03.03.o13 | Apply | Button |  | — | Applies updated VPN details. | After making updates. |
| WB.03.03.o14 | Connect | Button |  | — | Connects to the selected VPN. | On the VPN settings page, next to the VPN connection to use. |
| WB.03.03.o15 | Connected | Connection status |  | — | Displayed underneath the VPN connection name when connected. | After a VPN connection is established. |

- Confirmations / errors: The VPN connection name displays Connected underneath it on the VPN settings page when connected.
- SKU / hardware / version conditions: The feature is not available in Windows 11 SE.
- Windows 10 differences (legacy): For Windows 10, after selecting Advanced options, the page says to choose the VPN connection and then select Advanced options; it does not describe the Windows 11 Edit, Save, or Apply controls.; The Windows 10 connection instructions describe connecting from the taskbar Network icon and may direct the user to the VPN section in Settings.; The Windows 10 path for adding a connection is Start > Settings > Network & Internet > VPN > Add a VPN connection.


### WB.03.04 — Network & internet > Mobile hotspot

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Settings > Network & internet > Mobile hotspot`

ms-settings URIs: `ms-settings:network-mobilehotspot`

Sources: [Official support article (T1)](https://support.microsoft.com/en-gb/windows/use-your-windows-device-as-a-mobile-hotspot-c89b0fad-72d5-41e8-f7ea-406ad9036b85)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.03.04.o01 | Share my internet connection from | Dropdown | The internet connection you want to share; the page does not list specific choices. | — | Selects the internet connection to share. | — |
| WB.03.04.o02 | Share over | Dropdown | Wi-Fi; Bluetooth | Wi-Fi is usually faster and the default. | Selects how to share the connection. | — |
| WB.03.04.o03 | Properties | Expandable section |  | — | Expand to access hotspot properties. | — |
| WB.03.04.o04 | Edit | Button |  | — | Opens editing for the network name, password, and network band. | Located in the Properties section. |
| WB.03.04.o05 | network name | Text field |  | — | Enter a new network name. | Available after selecting Edit. |
| WB.03.04.o06 | password | Text field |  | — | Enter a new password. | Available after selecting Edit. |
| WB.03.04.o07 | network band | Setting |  | — | Enter or choose a network band; the page does not specify available choices. | Available after selecting Edit. |
| WB.03.04.o08 | Save | Button |  | — | Saves the edited network name, password, and network band. | After editing hotspot properties. |
| WB.03.04.o09 | Mobile hotspot | Toggle | On; off | — | Turns the mobile hotspot on or off. | At the top of the page. |
| WB.03.04.o10 | Go to Settings | Context-menu action |  | — | Opens Settings to view or manage hotspot settings, such as the network password, or to set up the hotspot. | Right-click or long-press the Mobile hotspot quick action. |

- SKU / hardware / version conditions: Remote hotspot activation requires the device to have cellular; both devices must have Bluetooth and be paired.; The page says a cellular connection can be shared and will use data from the device’s data plan.
- Windows 10 differences (legacy): The Windows 10 connection selector is labeled “Share my Internet connection from”.; The Windows 10 instructions describe connecting with the network name and password, but do not mention scanning a QR code.; The Windows 10 instructions say to select Edit, enter a new network name and password, and select Save; they do not mention a network band.; The Windows 10 instructions use the path Start > Settings > Network & Internet > Mobile hotspot.; The Windows 10 toggle is labeled “Share my Internet connection with other devices”.


### WB.03.05 — Network & internet > Airplane mode

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Network & internet > Airplane mode`; `Network & internet > Airplane mode`; `Network & Internet settings (mentioned; no specific path to the Airplane mode page is provided)`; `Settings > Network & Internet > Airplane mode settings (the user says to select the “>” at the far right of the “Airplane” mode selection line).`

ms-settings URIs: `ms-settings:network-airplanemode`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/experience/connectivity-networking/essential-network-settings-and-tasks-in-windows); [Community Q&A on Microsoft Learn (T5)](https://learn.microsoft.com/en-us/answers/questions/3911190/what-is-the-purpose-of-flight-mode-in-windows-11-i?page=1) (dated 2025-01-29); [Community Q&A on Microsoft Learn (T5)](https://learn.microsoft.com/en-us/answers/questions/4647827/ms-windows-11-airplane-mode-still-allows-google-br) (dated 2023-12-06)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.03.05.o01 | Airplane mode | Toggle | On; Off | — | Turns wireless communications off or on. Windows remembers whether Wi-Fi or Bluetooth was on the last time Airplane mode was used. To have either remain off the next time, turn it off while Airplane mode is on; it will also stay off after exiting Airplane mode. | — |
| WB.03.05.o02 | Wi-Fi | Setting/toggle |  | — | Can be manually turned on or off while Airplane Mode is active, according to the answers. | The answer says to turn it off manually in Network & Internet settings or Action Center if all wireless communications are to be off. |
| WB.03.05.o03 | Bluetooth | Setting/toggle |  | — | Can be manually turned on or off while Airplane Mode is active, according to the answers. | The answer says to turn it off manually in Network & Internet settings or Action Center if all wireless communications are to be off. |
| WB.03.05.o04 | WIFI | On/off option; the page does not specify its exact UI control type. | On or off | Not stated. The user says they initially found WIFI was still on, but also says they do not know the default. | The user reports that this setting determines whether WIFI is turned off when Airplane mode is enabled, and that the setting is remembered for later uses of Airplane mode. | Applies when Airplane mode is enabled. The user reports that after configuring WIFI to turn off, they had to reconnect to WIFI manually after leaving Airplane mode. |

- Confirmations / errors: The answers describe Airplane Mode as intended to disable wireless communications, but do not provide official page documentation or verify the reported behavior.; The questioner reports that Wi-Fi and Bluetooth remain connected and can connect to new networks or devices while Airplane Mode is enabled.; The user reports that internet access continued while Airplane mode was enabled and WIFI remained on, but does not establish why it happened or what the default setting is.; These details come from a user's account in a community thread, not an authoritative confirmation of Windows behavior.
- SKU / hardware / version conditions: Cellular connectivity is discussed as applying to devices with cellular capability; the questioner says their PC is not SIM-compatible.; The questioner identifies their device as a Dell Inspiron laptop running Windows 11 Home and says WIFI is their only internet connection. No specific model or hardware version is given.


### WB.03.06 — Network & internet > Proxy

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Settings > Network & internet > Proxy`

ms-settings URIs: `ms-settings:network-proxy`

Sources: [Official support article (T1)](https://support.microsoft.com/en-US/Windows/Experience/Connectivity-Networking/use-a-proxy-server-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.03.06.o01 | Automatic proxy setup | section |  | — |  | — |
| WB.03.06.o02 | Automatically detect settings | toggle | On or off | — | Turn on to have Windows detect proxy server settings automatically. | — |
| WB.03.06.o03 | Use setup script | button | Set up | — | Opens the Edit setup script dialog. | Use when you or your organization uses a setup script. |
| WB.03.06.o04 | Edit setup script | dialog |  | — | Turn on Use setup script, enter the script address, then select Save. | Opened by selecting Set up next to Use setup script. |
| WB.03.06.o05 | script address | text field |  | — | Enter the setup script address. | In the Edit setup script dialog, when using a setup script. |
| WB.03.06.o06 | Save | button |  | — | Saves the setup script settings. | In the Edit setup script dialog. |
| WB.03.06.o07 | Manual proxy setup | section |  | — |  | — |
| WB.03.06.o08 | Use a proxy server | button | Set up | — | Opens the Edit proxy server dialog. | Under Manual proxy setup. |
| WB.03.06.o09 | Edit proxy server | dialog |  | — | Turn on Use a proxy server; enter the proxy details and select Save. | Opened by selecting Set up next to Use a proxy server. |
| WB.03.06.o10 | Proxy IP address | text field |  | — | Enter the proxy server name or IP address. | In the Edit proxy server dialog, when using a proxy server. |
| WB.03.06.o11 | Port | text field |  | — | Enter the proxy server port; it is optional. | In the Edit proxy server dialog, when using a proxy server. |
| WB.03.06.o12 | exception list | text field |  | — | Enter website address endings to bypass the proxy. Separate multiple addresses with semicolons. The page gives .contoso.com* and *.adatum.com* as examples and says * is a wildcard. | In the Edit proxy server dialog, when using a proxy server. |
| WB.03.06.o13 | Don’t use the proxy server for local (intranet) addresses | checkbox | Selected or cleared | — | Select or clear the checkbox. The page recommends selecting it unless your organization requires the proxy server to be used for intranet addresses. | In the Edit proxy server dialog. |



### WB.03.07 — Network & internet > Dial-up

Status: PARTIAL. Official source confirms the page exists but documents few or none of its controls.

Navigation paths (as stated in sources): `Settings > Network and Internet`; `Settings menu`; `Settings view`

ms-settings URIs: `ms-settings:network-dialup`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/accessibility/windows/understand-and-explore-windows-settings); [Community Q&A on Microsoft Learn (T5)](https://learn.microsoft.com/en-us/answers/questions/4144597/is-there-a-way-to-add-a-dial-up-connection-to-the) (dated 2023-06-26)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.03.07.o01 | dial-Up connections | Not specified |  | — | The article says dial-up connections can be configured, edited, and managed directly within the Settings app. | — |
| WB.03.07.o02 | Connect | Button |  | — | Clicking it opens a credentials pop-up and allows the user to connect. | The dial-up connection has been created. |

- Confirmations / errors: Opening a desktop shortcut to the dial-up connection shows the available networks pop-up rather than the credentials pop-up.; The page says dial-up connections are no longer supported in Quick access in Windows 11.; The user says connection status can only be accessed within the Settings window.
- Keyboard shortcuts: A suggested workaround is to create a desktop shortcut from Control Panel > Network and Internet > Network and Sharing Center > Change Adapter Settings. The user reports that opening it does not provide the same connection experience as Settings.; Backspace; Ctrl+E; Down arrow; Enter; Tab; Up arrow; Windows + I
- Windows 10 differences (legacy): The Windows 10 section also says dial-up connections can be configured and edited in Settings; it states no difference from Windows 11 for this feature.; The user says that in Windows 10, a created dial-up connection appeared above the wireless settings, where it could be selected to connect; a pop-up showed credentials and connection status.


### WB.03.08 — Network & internet > Advanced network settings

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Network & internet > Advanced network settings > Network reset`

ms-settings URIs: `ms-settings:network-advancedsettings`, `ms-settings:network-status`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/experience/connectivity-networking/fix-ethernet-connection-problems-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.03.08.o01 | Advanced network settings | Settings page link |  | — | Opens the advanced network settings, where the page directs users to select Network reset. | — |
| WB.03.08.o02 | Network reset | Settings page |  | — | Removes installed network adapters and their settings. After the PC restarts, the adapters are reinstalled and their settings are set to defaults. | The page says to use network reset as a last step if earlier steps do not help. |
| WB.03.08.o03 | Reset now | Button |  | — | Starts network reset; the page says to wait for the PC to restart and then check whether the problem is fixed. | — |
| WB.03.08.o04 | Yes | Confirmation button |  | — | Confirms the network reset. | Shown after selecting Reset now. |

- Confirmations / errors: Select Yes to confirm the network reset.
- Restart / sign-out: After network reset, wait for the PC to restart.; Network reset might set known network connections to a public network profile. The page refers to Essential network settings and tasks for using a private network profile.; You might need to reinstall and set up networking software such as VPN client software or virtual switches from Hyper-V.
- Windows 10 differences (legacy): For Windows 10, the page gives the path Settings > Network & internet > Status > Network reset, rather than Advanced network settings > Network reset.

Preview/independent evidence (26H2-era):

- [Community Q&A on Microsoft Learn (T5)](https://learn.microsoft.com/en-us/answers/questions/5830890/wifi-related-issue) — applies to: Windows 11; a commenter states their version is 10.0.26200, Build 26200. The page does not say the instructions are specific to that build..
  - Network reset (Select “Reset now” and confirm.). Removes and reinstalls all network adapters and resets their settings to defaults. The PC restarts. VPN clients or virtual switches may need to be reinstalled or reconfigured, and known networks may be set to Public profile.
- [Community Q&A on Microsoft Learn (T5)](https://learn.microsoft.com/en-us/answers/questions/5850049/how-to-restore-ethernet-connection-after-windows-u) — applies to: Windows 11; no build or specific version stated..
  - Network reset (Settings option). Resets network settings. The AI answer says it removes and reinstalls all network adapters and resets network settings to defaults.
  - Reset now (Button). Starts the network reset.
  - Yes (Confirmation option). Confirms the network reset.


### WB.03.09 — Network & internet > Data usage

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `البدء > الإعدادات > الشبكة & حالة > الإنترنت > استخدام البيانات`

ms-settings URIs: `ms-settings:datausage` (not in MS Learn table — unverified)

Sources: [Official support article (T1)](https://support.microsoft.com/ar-sa/windows/%D8%AA%D8%B9%D9%8A%D9%8A%D9%86-%D8%AD%D8%AF-%D8%A8%D9%8A%D8%A7%D9%86%D8%A7%D8%AA%D9%83-031dcc15-fa0f-ad39-8e60-634500585630); [Official support article (T1)](https://support.microsoft.com/en-sg/help/10739/why-is-my-internet-connection-so-slow)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.03.09.o01 | استخدام البيانات | رابط/صفحة |  | — | يفتح إعدادات استخدام البيانات للشبكة. | ضمن الشبكة التي تتصل بها. |
| WB.03.09.o02 | اختيار الشبكة | قائمة اختيار | الشبكة التي تريد تعيين حد بيانات لها؛ لا تذكر الصفحة أسماء الخيارات. | — | يحدد الشبكة التي سيُضبط لها حد البيانات. | — |
| WB.03.09.o03 | إدخال الحد | زر/إجراء |  | — | يتيح إدخال حد بيانات للشبكة المختارة. | بعد اختيار الشبكة. |
| WB.03.09.o04 | نوع الحد | قائمة اختيار | أنواع الحدود لخطة البيانات؛ لا تحدد الصفحة الخيارات المتاحة. | — | يحدد نوع الحد، ثم تُدخل معلومات أخرى متعلقة بالحد. | ضمن «إدخال الحد». |
| WB.03.09.o05 | حفظ | زر |  | — | يحفظ حد البيانات والمعلومات المدخلة. | بعد إدخال معلومات الحد. |
| WB.03.09.o06 | data limit | Not stated | Not stated | Not stated | The page says that setting a data limit in Data usage settings for a cellular data connection can help Windows keep you under the limit and set the metered connection setting for you. | Applies to a cellular data connection. |

- Windows 10 differences (legacy): يذكر النص الخاص بذلك المسار أن Windows يُعلمك عند الاقتراب من الحد وعند تجاوزه؛ أما نص Windows 11 فيذكر الإشعار عند الاقتراب من الحد فقط.; يعرض المسار الآخر: «بدء» > «الإعدادات» > «الشبكة & الإنترنت»، ثم اختيار «استخدام البيانات» للشبكة المتصلة. بعد ذلك يُختار «إدخال حد»، ثم «نوع الحد» وتُدخل معلومات الحد ويُختار «حفظ».


### WB.03.10 — Network & internet > DNS over HTTPS

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Network & internet > Wi-Fi > Manage known networks > [network] > IP assignment > Edit > Edit IP settings`; `Network & internet > Ethernet > [connected Ethernet network] > IP assignment > Edit > Edit IP settings`

Sources: [Official support article (T1)](https://support.microsoft.com/en-US/Windows/Experience/Connectivity-Networking/essential-network-settings-and-tasks-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.03.10.o01 | DNS over HTTPS | Setting selector | Off; On (automatic template); On (manual template) | — | Choose whether DNS queries to the specified DNS server or servers use an encrypted or unencrypted connection. Off sends queries unencrypted in plaintext over HTTP. On (automatic template) encrypts queries and uses the default settings for the automatic template or tries to discover them automatically. On (manual template) encrypts queries and uses the settings entered in DNS over HTTPS template. | Shown while specifying IPv4 or IPv6 settings manually. The page says this setting is not available in Windows 10. |
| WB.03.10.o02 | DNS over HTTPS template | Text box |  | — | Enter the settings used for DNS over HTTPS when On (manual template) is selected. | Applies when DNS over HTTPS is set to On (manual template). |
| WB.03.10.o03 | Fallback to plaintext | Toggle | On; Off | — | When on, a DNS query is sent unencrypted if it can’t be sent over HTTPS. When off, a DNS query won’t be sent if it can’t be sent over HTTPS. | Available when DNS over HTTPS is set to either automatic or manual template. |
| WB.03.10.o04 | Save | Button |  | — | Saves the settings when you’re done. | Appears after configuring the manual IP settings. |

- Windows 10 differences (legacy): DNS over HTTPS setting is not available in Windows 10.


### WB.03.11 — Network & internet > Cellular

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Network & internet > Cellular`; `Start > Settings > Network & internet > Cellular > Use this SIM for cellular data`; `Start > Settings > Network & internet > Cellular > Let Windows keep me connected`; `Start > Settings > Network & internet > Cellular > Metered connection`; `Start > Settings > Network & internet > Cellular > Data roaming options`

ms-settings URIs: `ms-settings:network-cellular`

Sources: [Official support article (T1)](https://support.microsoft.com/en-US/Windows/Experience/Connectivity-Networking/cellular-settings-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.03.11.o01 | Use this SIM for cellular data | SIM selection setting | Physical SIM card slot or eSIM. | — | Determines which SIM slot or cellular network connection is used for cellular data. | Appears only on Windows 11 devices that have both a SIM and an eSIM. |
| WB.03.11.o02 | Let Windows keep me connected | Toggle | On or off. | — | When on, Windows automatically connects to cellular data when another kind of network is not connected. When off, connect manually. | — |
| WB.03.11.o03 | Metered connection | Setting |  | Cellular data connections are set as metered by default. | A metered connection can cause some apps to work differently to reduce data usage, and some Windows updates will not install automatically. A data limit in Data usage settings can set this for you. | Changing this setting may be unavailable; the page refers to Metered connections in Windows for help. |
| WB.03.11.o04 | Data roaming options | Setting | Don't roam; allow roaming. | — | Determines whether cellular data stays on outside the mobile operator's network. Allowing roaming lets the device use cellular data in a roaming area and may incur additional charges. | — |
| WB.03.11.o05 | Use cellular whenever Wi-Fi is poor | Toggle | On or off. | — | When on, Windows can use cellular when Wi-Fi is slow or cellular would be faster. When off, Windows uses cellular only when Wi-Fi is unavailable. Cellular use draws on the data plan and may incur charges. | — |
| WB.03.11.o06 | Cellular data and apps | Settings sub-page |  | — | Contains cellular data and app controls. | — |
| WB.03.11.o07 | Choose apps that can use your cellular data | Settings page/control | Turn cellular data off for all apps or for an individual app listed under this setting. | — | Controls which apps can use cellular data. | The Windows Subsystem for Android™, the Amazon Appstore, and installed apps might still use cellular data even if they are not listed on the Cellular data and apps Settings page. |
| WB.03.11.o08 | Let apps use my cellular data | Toggle | On or off. | — | When off, apps cannot use cellular data and send or receive data only when connected to another kind of network. | Located on the Cellular data screen. |
| WB.03.11.o09 | Cellular network selection | Network selection setting | Automatic; Search, then select another cellular network. | Automatic. | Determines which cellular network connection is used when roaming. Search can be used if the selected network is unavailable. | Appears when roaming. |
| WB.03.11.o10 | Mobile operator settings | Settings sub-page |  | — | Contains cellular network, APN, SIM security, and connection information settings described on the page. | — |
| WB.03.11.o11 | Active network | Display |  | — | Displays the name of the cellular network in use. | — |
| WB.03.11.o12 | APN settings | Settings section |  | The internet APN is usually set automatically. | An APN is the address the device uses to connect to the internet over cellular data. A new internet APN can be entered if cellular data is not working and the device cannot connect to the internet. | — |
| WB.03.11.o13 | Add APN | Button/action |  | — | Opens the controls for adding an APN profile. | Found under Mobile operator settings. |
| WB.03.11.o14 | Profile name | Text box |  | — | Enter a name for the APN profile. | Shown when adding an APN. |
| WB.03.11.o15 | APN | Text box |  | — | Enter the address for the APN to use. | Shown when adding an APN. |
| WB.03.11.o16 | User name | Text box |  | — | Enter the user name for the mobile account. | Shown when adding an APN. |
| WB.03.11.o17 | Password | Text box |  | — | Enter the password for the mobile account. | Shown when adding an APN. |
| WB.03.11.o18 | Type of sign-in info | Dropdown | Select the authentication method used. | — | Specifies the authentication method. | Shown when adding an APN. |
| WB.03.11.o19 | IP type | Dropdown | Select the type of IP address to use. | — | Specifies the IP address type. | Shown when adding an APN. |
| WB.03.11.o20 | APN type | Dropdown | Internet. | — | Select Internet for the APN type. | Shown when adding an APN. |
| WB.03.11.o21 | Apply this profile | Checkbox | Selected or not selected. | — | If selected, the APN profile is used right after it is saved. | Shown when adding an APN. |
| WB.03.11.o22 | Save | Button |  | — | Saves the APN profile. | Used when adding an APN. |
| WB.03.11.o23 | OK | Button |  | — | Selected after Save when adding an APN; also used in SIM PIN procedures. | — |
| WB.03.11.o24 | Properties | Settings section |  | — | Displays information about the SIM and cellular data connection, such as the IMEI number. | — |
| WB.03.11.o25 | Copy | Button |  | — | Copies the displayed SIM and cellular connection information for pasting elsewhere. | Available in Properties. |
| WB.03.11.o26 | Use SIM PIN | SIM security setting |  | — | Sets a PIN for the SIM. After setup, the SIM PIN is requested when connecting. | Found under Security. The device must be restarted for the cellular data connection to be locked with a SIM PIN. |
| WB.03.11.o27 | Remove SIM PIN | Button/action |  | — | Removes the SIM PIN after entering the current SIM PIN and selecting OK. | Appears when using a SIM PIN; found under Security. |
| WB.03.11.o28 | Change SIM PIN | Button/action |  | — | Changes the PIN using the Current SIM PIN, New SIM PIN, and Confirm new SIM PIN fields, then selecting OK. | Appears when using a SIM PIN; found under Security. |
| WB.03.11.o29 | Current SIM PIN | Text box |  | — | Enter the current SIM PIN. | Shown when changing the SIM PIN. |
| WB.03.11.o30 | New SIM PIN | Text box |  | — | Enter the new SIM PIN. | Shown when changing the SIM PIN. |
| WB.03.11.o31 | Confirm new SIM PIN | Text box |  | — | Enter the new SIM PIN again. | Shown when changing the SIM PIN. |
| WB.03.11.o32 | Unblock SIM PIN | Button/action |  | — | Unblocks the SIM by entering the PIN Unblocking Key (PUK) code. | Appears when using a SIM PIN after the incorrect PIN is entered three times; found under Security. |
| WB.03.11.o33 | PUK code | Text box |  | — | Enter the PUK code provided by the mobile operator to unblock the SIM. | Used when unblocking a SIM PIN. |

- Confirmations / errors: Entering an incorrect PUK code too many times permanently blocks the SIM; a new SIM card is then required from the mobile operator.; Entering an incorrect SIM PIN three times blocks the SIM until it is unblocked with a PUK code.; If the selected cellular network is unavailable, use Search and select another cellular network.; When prompted to enter a SIM PIN, enter it; if using it for the first time, enter the default SIM PIN. The article says to check the mobile operator's website if the default PIN is unknown.
- Restart / sign-out: Restart the Windows 11 device for the cellular data connection to be locked with a SIM PIN.
- SKU / hardware / version conditions: Cellular data requires a SIM card or eSIM, or an external cellular modem/mobile broadband device, and a data plan from a mobile operator.; Cellular network selection appears when roaming.; Settings vary by Windows 11 device model and mobile operator.; Use this SIM for cellular data appears only on Windows 11 devices that have both a SIM and an eSIM.


## WB.04 — Personalization


### WB.04.01 — Personalization > Background

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Settings > Personalization > Background`; `Right-click the desktop > Personalize > Background`

ms-settings URIs: `ms-settings:personalization-background`

Sources: [Official support article (T1)](https://support.microsoft.com/help/17144/change-the-desktop-background-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.04.01.o01 | Personalize your background | Dropdown | Picture; Solid Color; Slideshow; Windows spotlight | — | Selects the type of desktop background. | — |
| WB.04.01.o02 | Picture | Background option | Default images; Browse | — | Selects a single image as the desktop background. An image file can also be set as the background by right-clicking it and selecting Set as desktop background; in Photos, use the three-dot menu > Set as > Background. | For multiple displays, the same picture can be used on all displays or a different picture can be set for each display. Per-monitor background pictures are not supported when using multiple desktops. |
| WB.04.01.o03 | Browse | Button |  | — | Selects an image from the device. | Available when Picture is selected. |
| WB.04.01.o04 | Solid Color | Background option | A single color | — | Sets a single color as the desktop background. | — |
| WB.04.01.o05 | Slideshow | Background option | A folder of images | — | Cycles through images from a selected folder. The interval for changing images and whether to shuffle pictures can be set. Right-click the desktop and select Next desktop background to move to the next picture. Each display shows a different picture from the selected folder. | Slideshow as a background is not supported when using multiple desktops. |
| WB.04.01.o06 | interval | Setting | How often the images change | — | Sets how often slideshow images change. | Available when Slideshow is selected. |
| WB.04.01.o07 | shuffle pictures | Setting |  | — | Sets whether slideshow pictures are shuffled. | Available when Slideshow is selected. |
| WB.04.01.o08 | Windows spotlight | Background option | A new image from around the world every day | — | Displays a new image each day. | — |
| WB.04.01.o09 | Fill | Image fit option |  | — | Enlarges the image to fill the screen; parts might be cropped. | For Picture or Slideshow. |
| WB.04.01.o10 | Fit | Image fit option |  | — | Fits the image within the screen while maintaining its aspect ratio; bars might appear if the aspect ratios do not match. | For Picture or Slideshow. |
| WB.04.01.o11 | Stretch | Image fit option |  | — | Stretches the image to cover the screen and may distort it if its aspect ratio differs from the screen's. | For Picture or Slideshow. |
| WB.04.01.o12 | Tile | Image fit option |  | — | Repeats the image across the screen. | For Picture or Slideshow. |
| WB.04.01.o13 | Center | Image fit option |  | — | Centers the image without resizing it; a border appears if the image is smaller than the screen. | For Picture or Slideshow. |
| WB.04.01.o14 | Span | Image fit option |  | — | Spans the image across multiple monitors. | For Picture or Slideshow. |
| WB.04.01.o15 | Next desktop background | Desktop context-menu command |  | — | Moves to the next slideshow picture. | When using Slideshow. |
| WB.04.01.o16 | Recent images | Image preview area |  | — | Right-click an image preview to choose Set for all desktops or a specific desktop. | When using multiple desktops. |
| WB.04.01.o17 | Set for all desktops | Context-menu command |  | — | Applies the selected background picture to all desktops. | When using multiple desktops. |
| WB.04.01.o18 | Set as desktop background | Image-file context-menu command |  | — | Sets the image as the desktop background. | Available by right-clicking an image file. |
| WB.04.01.o19 | Set as | Photos menu command | Background | — | Sets the viewed picture as the desktop background. | In the Photos app, use the three-dot menu. |



### WB.04.02 — Personalization > Colors

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Settings > Personalization > Colors`

ms-settings URIs: `ms-settings:colors`, `ms-settings:personalization-colors`

Sources: [Official support article (T1)](https://support.microsoft.com/en-US/Windows/Experience/personalization/personalize-your-colors-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.04.02.o01 | Choose your mode | Selection | Light, Dark, Custom | — | Selects the Windows color mode. | — |
| WB.04.02.o02 | Automatic | Selection | Automatically pick an accent color from your background | — | Lets Windows pick an accent color from the background. | — |
| WB.04.02.o03 | Manual | Selection | Recent colors, Windows colors, View colors | — | Lets you manually select an accent color. | — |
| WB.04.02.o04 | Recent colors | Color choices |  | — | Select a color from the recent colors. | Available after selecting Manual. |
| WB.04.02.o05 | Windows colors | Color choices |  | — | Select a color from the Windows colors. | Available after selecting Manual. |
| WB.04.02.o06 | View colors | Button |  | — | Opens a more detailed option for selecting a color. | Available after selecting Manual. |
| WB.04.02.o07 | Show accent color on Start and taskbar | Toggle |  | — | Applies the selected accent color to the Start menu, taskbar, and action center. | Available only when the Windows color mode is Dark or Custom. |
| WB.04.02.o08 | Show accent color on title bars and window borders | Toggle |  | — | Applies the selected accent color to window title bars and borders. | — |
| WB.04.02.o09 | Transparency effects | Toggle | Enabled or disabled | — | Makes some window backgrounds and surfaces appear slightly transparent with an acrylic blur. | — |
| WB.04.02.o10 | Contrast themes | Setting or sub-page |  | — | Enhances the visibility of text and UI elements using distinct color combinations that can be customized. | — |
| WB.04.02.o11 | Color filters | Setting or sub-page |  | — | Changes the screen's color palette to help distinguish between things that differ only by color. | — |
| WB.04.02.o12 | Open Color settings | Link |  | — | Opens Color settings. | — |
| WB.04.02.o13 | Change color contrast in Windows | Link |  | — | Links to information about changing color contrast in Windows. | — |
| WB.04.02.o14 | Use color filters in Windows | Link |  | — | Links to information about using color filters in Windows. | — |



### WB.04.03 — Personalization > Themes

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Settings > Personalization > Themes`; `Settings > Personalization > Themes > Desktop icon settings`

ms-settings URIs: `ms-settings:themes`

Sources: [Official support article (T1)](https://support.microsoft.com/en-US/Windows/experience/personalization/customize-the-desktop-icons-in-windows); [Official support article (T1)](https://support.microsoft.com/en-us/windows/experience/personalization/personalize-your-windows-experience-with-themes); [Official support article (T1)](https://support.microsoft.com/windows/windows-designs-94880287-6046-1d35-6d2f-35dee759701e)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.04.03.o01 | Desktop icon settings | sub-page | This PC; Recycle Bin; Network; User's Files | — | Select the icons to show on the desktop, then select OK. | — |
| WB.04.03.o02 | Allow themes to change desktop icons | option (control type not specified) | Allow or prevent themes from changing common desktop shortcut icons, including This PC, User's Files, Network, and Recycle Bin. | By default, themes are allowed to change these icons. | Allows or prevents themes from changing the icons used for common desktop shortcuts. | — |
| WB.04.03.o03 | OK | button |  | — | Confirms the selected desktop icons. | In Desktop icon settings. |
| WB.04.03.o04 | Browse themes | Link | Opens the Microsoft Store to browse and download themes. | Not stated. | Browse for themes to download from the Microsoft Store. | Not stated. |
| WB.04.03.o05 | Theme | Selectable theme | Default themes or themes browsed from the Microsoft Store. | Not stated. | Selecting a theme applies it to the desktop. | Not stated. |
| WB.04.03.o06 | Background | Customization option | Not stated. | Not stated. | Customizes the theme’s background. | Not stated. |
| WB.04.03.o07 | Color | Customization option | Not stated. | Not stated. | Customizes the theme’s color. | Not stated. |
| WB.04.03.o08 | Sounds | Customization option | Not stated. | Not stated. | Customizes the theme’s sounds. | Not stated. |
| WB.04.03.o09 | Mouse cursor | Customization option | Not stated. | Not stated. | Customizes the mouse cursor for the theme. | Not stated. |
| WB.04.03.o10 | Save | Button | Not stated. | Not stated. | Saves the customized theme after you provide a name. | Available after customizing theme settings. |
| WB.04.03.o11 | Save theme for sharing | Context-menu command | Not stated. | Not stated. | Creates a .deskthemepack file that can be shared. | Right-click a saved theme. |
| WB.04.03.o12 | Delete | Context-menu command | Not stated. | Not stated. | Deletes a theme. | Right-click a saved theme; default Windows themes cannot be deleted. |
| WB.04.03.o13 | Change color contrast in Windows | Link | Not stated. | Not stated. | Links to information about changing color contrast. | Not stated. |
| WB.04.03.o14 | Use color filters in Windows | Link | Not stated. | Not stated. | Links to information about using color filters. | Not stated. |

- Confirmations / errors: The page says default Windows themes cannot be deleted.; You can't delete the default Windows themes.
- Windows 10 differences (legacy): The Windows 10 section additionally gives a shortcut labeled “Themes” for opening Themes settings.; The Windows 10 section also gives the shortcut label “Themes” for opening Themes settings.


### WB.04.04 — Personalization > Dynamic Lighting

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Personalization > Dynamic Lighting`

ms-settings URIs: `ms-settings:personalization-lighting`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/hardware/input-devices/control-dynamic-lighting-devices-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.04.04.o01 | Open Dynamic Lighting settings | Link |  | — | Opens Dynamic Lighting settings. | — |
| WB.04.04.o02 | Use Dynamic Lighting on my devices | Toggle | On; Off | — | Turns Dynamic Lighting on or off. When off, devices use their default non-Dynamic Lighting behavior. | — |
| WB.04.04.o03 | Compatible apps in the foreground always control lighting | Toggle | On | — | When on, an active application can take control of device lighting. Its effects can override background applications; when it is minimized or closed, background application lighting resumes control. | — |
| WB.04.04.o04 | Background light control | Section with a prioritized app list | Registered ambient background controllers; drag an app to change its priority | — | Prioritizes installed apps that have registered as ambient background controllers. An app at the top of the list is prioritized to control devices before other listed apps. | Ambient background settings are tied to a device and the port it is connected to. Connecting the device to a different USB port makes it appear as a different device. |
| WB.04.04.o05 | Brightness | Slider; button | Brightness level; Reset for all devices | — | Sets LED brightness. Selecting Reset for all devices resets brightness to a default value. | — |
| WB.04.04.o06 | Effects | Effect and color selector; button | Colors and effects; Reset for all devices | — | Selects colors and effects. Selecting Reset for all devices resets effects to the devices' default behaviors. | — |
| WB.04.04.o07 | Dynamic lighting devices | Link |  | — | See the list of compatible devices. | — |
| WB.04.04.o08 | Dynamic Lighting - UWP applications | Link |  | — | Provides more information about ambient controllers. | — |

- Confirmations / errors: If the Brightness or Effects sliders do not change device behavior, a higher-priority background RGB controller app may be running. Check Background light control and, if needed, drag that controller to the top of the list.
- SKU / hardware / version conditions: Ambient background settings are tied to the device and its USB port; plugging it into a different port makes it appear as a different device.; Compatible devices appear as cards along the top of the page when connected. Selecting a card configures that device; its individual settings override global settings for that device.; Global settings are configured after the device cards and affect all connected devices.


### WB.04.05 — Personalization > Lock screen

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Personalization > Lock screen`

ms-settings URIs: `ms-settings:lockscreen`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/experience/personalization/customize-the-lock-screen-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.04.05.o01 | Personalize your lock screen | Selection menu | Windows spotlight; Picture; Slideshow | — | Choose Windows spotlight for rotating images, tips, tricks, and notifications; Picture for a single image; or Slideshow for images from device folders. | — |
| WB.04.05.o02 | Get Fun Facts, Tips, Tricks, and More on Your Lock Screen | Option | Enabled or not enabled | — | Enables additional content from Microsoft on the lock screen. | — |
| WB.04.05.o03 | Make the lock screen image react when I move my PC | Toggle | On; Off | — | When enabled, the image slightly pans or moves in response to the device’s physical movement. | — |
| WB.04.05.o04 | Show the lock screen background picture on the sign-in screen | Toggle | On; Off | — | Uses the same background image for the lock screen and sign-in screen. | — |
| WB.04.05.o05 | Widgets | Widget list and toggle | On; Off | — | Review lock-screen widgets; turning Widgets Off hides them. Drag a widget higher in the list to make it appear earlier on the lock screen. | — |
| WB.04.05.o06 | more options | Button for each widget | Not stated | — | Remove a widget or customize it, if its provider supports customization. | Customization depends on whether the widget provider supports it. |
| WB.04.05.o07 | Add widget | Button | Catalog of available widgets | — | Opens the available-widget catalog. Only widgets that support a small size can currently be added to the lock screen. | — |

- Windows 10 differences (legacy): The Windows 10 section describes lock-screen status settings: "Choose one app to show detailed status on the lock screen" lets you select an app (for example, Weather and more) or None; "Choose which apps show quick status on the lock screen" lets you select apps using plus icons.; The Windows 10 section says the Weather and more app’s widgets list cannot currently be customized. It says its location, language, region, and temperature units can be configured through Widgets, and references "Configure Widgets settings".; The Windows 10 section uses the setting label "Background" instead of "Personalize your lock screen" for the Windows spotlight, Picture, and Slideshow choices.


### WB.04.06 — Personalization > Text input

Status: PARTIAL. Official source confirms the page exists but documents few or none of its controls.

Navigation paths (as stated in sources): `Settings > Personalization`

ms-settings URIs: `ms-settings:personalization-textinput`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/accessibility/windows/understand-and-explore-windows-settings)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.04.06.o01 | Text Input | Settings area (control type not stated) |  | — | Personalize the on-screen keyboard and typing insights to optimize the typing experience. | — |

- Keyboard shortcuts: Backspace; Ctrl+E; Down arrow; Enter; Tab; Up arrow; Windows + I

Preview/independent evidence (26H2-era):

- [Independent (T4)](https://techdows.com/windows-11-personalization-settings-will-help-you-customize-your-pc.html) — applies to: Windows 11; no specific version or build is stated..
  - Touch keyboard themes (Theme selection on the Text input page): One of the listed themes; the custom theme option — default: Not stated. Applies a theme to the touch keyboard. The custom theme option lets you create a theme with a background image.
  - Resize the keyboard (Setting on the Text input page): Not stated — default: Not stated. Changes the touch keyboard size.
  - Touch keyboard layouts (Keyboard settings icon in the touch keyboard): Default, Split, Small, Traditional, Handwriting panel — default: Not stated. Switches the touch keyboard layout.
  - Voice typing launcher (Voice typing settings): On or off (the article says it can be turned on) — default: Off. Enables the voice typing launcher.
  - Requirement/rollout: No Copilot+ PC, Insider channel, region, or SKU requirement is stated for Text input.
  - Requirement/rollout: Split and Small touch keyboard layouts are not available in every language.
- [Official documentation (T1)](https://learn.microsoft.com/en-us/windows/apps/develop/settings/settings-windows-11) — applies to: Windows 11, as identified by the page title; no specific build or version is stated..
  - Text Input (Section heading; the page does not provide a Settings navigation path or exact in-app control labels.). Described as helping choose a theme for touch keyboard, voice typing, emoji and more, and input method editors.
  - IsKeyBackgroundEnabled (Registry value (REG_DWORD) under HKCU\Software\Microsoft\TabletTip\1.7): 0 or 1. Indicates if key background is enabled.
  - KeyLabelSize (Registry value (REG_DWORD) under HKCU\Software\Microsoft\TabletTip\1.7): Minimum 0; maximum 2. Key label size.
  - UserKeyboardScalingFactor (Registry value (REG_DWORD) under HKCU\Software\Microsoft\TabletTip\1.7): Minimum 1; maximum 1000. User keyboard scaling factor.
  - SelectedThemeIndex (Registry value (REG_DWORD) under HKCU\Software\Microsoft\TabletTip\1.7): Minimum 0; maximum 15. Selected theme index.
  - SelectedThemeName (Registry value (REG_SZ) under HKCU\Software\Microsoft\TabletTip\1.7): "LightTheme", "DarkTheme", "ColorPopTheme", "BlackWhiteTheme", "PoppyRedTheme", "IceBlueTheme", "PlatinumTheme", "TangerineTidesTheme", "LilacRiverTheme", "SilkyDawnTheme", "IndigoBreezeTheme", "PinkBlueTheme", "GreenPurpleTheme", "PinkOrangeTheme", "CustomTheme". Selected theme name.
  - ThemeDataVersion (Registry value (REG_DWORD) under HKCU\Software\Microsoft\TabletTip\1.7): 2. Theme data version.


### WB.04.07 — Personalization > Start

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Start settings > Folders`; `Start > right-click the bottom of the Start menu > Start settings`; `Settings > Personalization > Start`

ms-settings URIs: `ms-settings:personalization-start`, `ms-settings:personalization-start-places`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/experience/personalization/customize-the-windows-start-menu); [Official support article (T1)](https://support.microsoft.com/en-au/windows/customize-the-windows-start-menu-fde6f576-0fc0-0813-6b0d-d3ec1d244c50); [Official documentation (T1)](https://learn.microsoft.com/en-us/windows/configuration/start/policy-settings?tabs=start&pivots=windows-11) (dated 2026-02-10)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.04.07.o01 | Start settings | Shortcut |  | — | Opens Start settings. | — |
| WB.04.07.o02 | Folders | Section | Documents, Downloads, Music, Pictures, Videos, Network, Personal folder | — | Toggle folders On to make them appear next to the Power button. | — |
| WB.04.07.o03 | different providers to Start Recommended | Toggle | On or off | — | Users can toggle providers off. If all recommended providers are toggled off, the sections collapse and are not shown. | Applies to recommended providers in Settings. |
| WB.04.07.o04 | More Pins | Layout option | More Pins; More Recommendations; Default | — | Prioritizes showing more pinned apps. | — |
| WB.04.07.o05 | More Recommendations | Layout option | More Pins; More Recommendations; Default | — | Shows more recommended items, such as recently added apps and files. | — |
| WB.04.07.o06 | Default | Layout option | More Pins; More Recommendations; Default | — | Balances between pinned apps and recommended items. | — |
| WB.04.07.o07 | Show recently added apps | Toggle | On; Off | — | Controls whether recently added apps are shown. | — |
| WB.04.07.o08 | Show most used apps | Toggle | On; Off | — | Controls whether most used apps are shown. | — |
| WB.04.07.o09 | Show recommended files in Start, recent files in File Explorer, and items in Jump Lists | Toggle | On; Off | — | Controls whether the named recommendations and recent items are shown. | — |
| WB.04.07.o10 | Show recommendations for tips, shortcuts, new apps, and more | Toggle | On; Off | — | Controls whether recommendations for tips, shortcuts, new apps, and more are shown. | — |
| WB.04.07.o11 | Show app list in Start | Settings option; control type not stated | On; Off | — | The article says setting this option to Off is equivalent to the policy's Collapse behavior. If the policy is not enabled or configured, the app list is visible by default and the user can change this option in Settings. | The Collapse and disable setting and Remove and disable setting policy choices disable the Settings option so users can't turn it On. |
| WB.04.07.o12 | Show app list in Start menu | Settings option; control type not stated | On or Off is not stated for this exact label | — | The article says the Collapse and disable setting and Remove and disable setting policy choices disable this option so users can't turn it On. | These behaviors are described under the Hide app list policy. |
| WB.04.07.o13 | Recently added | Corresponding Settings option; control type and exact UI label are not stated | Not stated | — | When Hide recently added apps is enabled, the Start menu doesn't display the Recently added list and the corresponding option in Settings is grayed out. When disabled or not configured, the list is displayed and the option can be configured. | Depends on the Hide recently added apps policy. |
| WB.04.07.o14 | Most used | Settings control for the list; control type and exact UI label are not stated | Users can turn display of the list on or off in Settings when the policy is Not Configured. | Not Configured is described as the default policy behavior; no default UI selection is stated. | The policy can force the list to Show or Hide, preventing users from changing that choice in Settings. When Not Configured, users can turn the list on or off in Settings. | The Show or hide "Most used" list from Start menu policy takes precedence over Hide frequently used apps. |
| WB.04.07.o15 | size of Start | Settings control; exact UI label and control type are not stated | The article names the policy values Start menu and full screen Start, but does not state the Settings control's choices. | — | When Force Start size is enabled and set to Start menu or full screen Start, users can't change the size of Start in Settings. When disabled or not configured, users can change it. | Depends on the Force Start size policy. |

- Confirmations / errors: The article describes policy effects on some Settings options but does not provide their full Settings-page labels or control types.; The article does not state the Personalization > Start navigation path.
- Windows 10 differences (legacy): Choose which folders appear on Start: select this option, then toggle On the folders to show next to the Power button.; Show app list in Start menu: toggle to On to show the app list.; The Windows 10 portion describes Start tiles rather than Windows 11 pins, including resizing tiles and organizing tiles into folders.; The Windows 10 portion says Start-menu color can be modified by changing Windows color mode or configuring accent colors, and links to “Personalize Your Colors in Windows.”; Use Start full screen: toggle to On to make Start full screen.; Windows 10 route: Settings > Personalization > Start > Choose which folders appear on Start. The page says to toggle On the folders to show next to the Power button.; Windows 10 route: Settings > Personalization > Start. It names the toggle “Show app list in Start menu” and says to toggle it On to hide the app list.; Windows 10 uses Settings > Personalization > Start.

Preview/independent evidence (26H2-era):

- [Official Insider/Experience blog (T1, preview-scoped)](https://blogs.windows.com/windows-insider/2026/05/15/improving-windows-quality-making-taskbar-and-start-more-personal/) — applies to: Windows 11; no specific version or build is stated..
  - Pinned (Section-level toggle): Show or hide. Controls whether the Pinned section appears in Start.
  - Recommended (Section-level toggle): Show or hide. Controls whether the Recommended section appears in Start. The article also says this section is being renamed to “Recent.”
  - All (Section-level toggle): Show or hide. Controls whether the All section appears in Start.
  - File recommendations (Separate control; exact UI label not stated): Enable or disable. Lets you disable file recommendations in Start without affecting recent files in other places, such as File Explorer and jump lists.
  - Start menu size (Size setting): Small or Large. Lets you choose a preferred Start menu size, intended to stay consistent across displays whenever possible.
  - Hide your name and profile picture (Option; exact UI label not stated): Hide or show. Hides the name and profile picture in Start for added privacy when sharing a screen, presenting, or streaming.
  - Requirement/rollout: The Start changes will roll out to Windows Insiders in the Experimental channel over the coming weeks.
  - Requirement/rollout: The article says several features are included in the current flight, but does not specify which Start changes are included.
  - Requirement/rollout: No Copilot+ PC, region, or SKU requirement is stated.
- [Official Insider/Experience blog (T1, preview-scoped)](https://blogs.windows.com/windows-insider/2025/06/09/announcing-windows-11-insider-preview-build-26200-5641-dev-channel/) — applies to: Windows 11 Insider Preview Build 26200.5641 (KB5060824), Dev Channel; Dev Channel builds are based on Windows 11, version 24H2..
  - All apps view (View options in the “All” section): Category, grid, or List view — default: Category view is the new default. The last-used view is remembered.. Category groups apps by category when a category has at least 3 apps; otherwise, apps remain in “Other.” Grid view is alphabetical and provides more horizontal space for scanning.
  - Responsive Start sections (Start menu layout adapts to screen size and available content): Larger devices: 8 columns of pinned apps, 6 recommendations, and 4 columns of categories. Smaller devices: 6 columns of pinned apps, 4 recommendations, and 3 columns of categories. — default: The Start menu is larger by default on larger-screen devices.. Pinned and Recommended sections can show more or less content. With only a few pins, Pinned shrinks to one row and other sections move up. The Recommended section collapses if there are no recommendations available.
  - Keep pins expanded (Setting in Settings; exact control label not stated): Not stated — default: Not stated. Users can choose to always have their pins expanded by default.
  - Show recently added apps (Toggle): On or off — default: Not stated. Turning it off removes recently added apps from recommendations in Start.
  - Show recommended files in Start… (Toggle): On or off — default: Not stated. Turning it off removes recommended files from Start.
  - Show websites from your browsing history (Toggle): On or off — default: Not stated. Turning it off removes websites from browsing history from Start recommendations.
  - Show recommendations for tips… (Toggle): On or off — default: Not stated. Turning it off removes recommendations for tips from Start.
  - Requirement/rollout: The new Start menu is gradually rolling out to Windows Insiders in the Dev Channel with the “get the latest updates as they are available” toggle turned on in Settings > Windows Update.
  - Requirement/rollout: The post does not state a Copilot+ PC, region, or SKU requirement for the new Start menu or its Personalization > Start settings.
  - Requirement/rollout: Windows Insider features may change or be removed and are not guaranteed to ship in a future Windows release.


### WB.04.08 — Personalization > Taskbar

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Right-click on the taskbar > Taskbar settings`; `Right-click on the clock on the taskbar > Adjust date and time`

ms-settings URIs: `ms-settings:taskbar`

Sources: [Official support article (T1)](https://support.microsoft.com/help/29280)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.04.08.o01 | Widgets | Toggle under Taskbar items | On; Off | — | Shows or hides Widgets from the taskbar. | — |
| WB.04.08.o02 | Search | Dropdown under Taskbar items | Search box; Search icon and label; Search icon only; Hide | — | Controls how Search appears on the taskbar, or hides it. | — |
| WB.04.08.o03 | Task view | Toggle under Taskbar items | On; Off | — | Shows or hides the Task view button from the taskbar. | — |
| WB.04.08.o04 | Pin to taskbar | Context-menu command |  | — | Pins an app to the taskbar. The page says this is available from Search or by right-clicking an app that is already open. | — |
| WB.04.08.o05 | Unpin from taskbar | Context-menu command |  | — | Removes an app from the taskbar. | — |
| WB.04.08.o06 | System tray icons | Icon choices in Taskbar settings | Pen Menu; Touch keyboard; Virtual Touchpad | — | Choose which of these icons appear in the system tray. | — |
| WB.04.08.o07 | Other system tray icons | Section with an option to enable the hidden icon menu and choices for which icons appear |  | — | Controls the hidden icon menu (also called the overflow area) and which icons can appear in the system tray. | — |
| WB.04.08.o08 | Show time and date in the System tray | Toggle | On; Off | — | Shows or hides the time and date in the system tray. | — |
| WB.04.08.o09 | Show seconds in system tray clock (uses more power) | Checkbox | Checked; unchecked | — | Shows seconds alongside hours and minutes in the system tray clock. | Described as available when time and date are shown. |
| WB.04.08.o10 | Taskbar alignment | Selection under Taskbar behaviors | Center; Left | The taskbar is centered by default. | Aligns taskbar icons in the center or on the left. | — |
| WB.04.08.o11 | Show badges on taskbar apps | Checkbox under Taskbar behaviors | Checked; unchecked | — | Shows or hides badges on taskbar app buttons. | — |
| WB.04.08.o12 | Show flashing on taskbar apps | Checkbox under Taskbar behaviors | Checked; unchecked | By default, an app icon flashes several times and stays highlighted until selected or opened. | Controls whether taskbar app icons flash to get attention. | — |
| WB.04.08.o13 | Share any window from my taskbar | Checkbox under Taskbar behaviors | Checked; unchecked | — | When in a Microsoft Teams meeting call, hovering over a running app can show a button to share its window. Hover over the window again to select Stop Sharing or another window. | The described sharing behavior is for a meeting call through Microsoft Teams. |
| WB.04.08.o14 | Show my taskbar on all displays | Checkbox under Taskbar behaviors | Checked; unchecked | — | Shows the taskbar on all connected displays. | Applies when multiple displays are connected. |
| WB.04.08.o15 | Automatically hide the taskbar | Checkbox under Taskbar behaviors | Checked; unchecked | — | Automatically hides the taskbar. Hover over or touch the bottom edge of the screen to see it. | — |
| WB.04.08.o16 | Optimize taskbar for touch interactions when this device is used as a tablet | Checkbox | Checked; unchecked | — | Turns the tablet-optimized taskbar on or off. | Only available on 2-in-1 devices with tablet functionality. |
| WB.04.08.o17 | Select the far corner of the taskbar to show the desktop | Option under Taskbar behaviors | Enabled; disabled | — | Controls whether selecting the far corner of the taskbar shows the desktop. | — |
| WB.04.08.o18 | Personalize Your Colors in Windows | Link |  | — | The page directs readers here to learn about changing the taskbar color through Windows color mode or accent colors. | — |
| WB.04.08.o19 | Change the taskbar alignment | Link |  | — | The page points to this for information about aligning the taskbar to the left. | — |

- Confirmations / errors: The page states that Windows 11 taskbar icons can be aligned in the center or on the left; it provides no setting to move the taskbar to the top or side of the screen, and says it is positioned at the bottom.
- Keyboard shortcuts: Windows key; Windows key + A; Windows key + N; Windows key + S; Windows key + Tab; Windows key + W
- SKU / hardware / version conditions: The tablet-optimized taskbar appears when the keyboard is disconnected or folded back.; The tablet-optimized taskbar setting is only available on 2-in-1 devices with tablet functionality.
- Windows 10 differences (legacy): Action center quick actions can be edited through Settings > Notifications & Actions > Edit your quick actions; select Add, rearrange actions by dragging, and select Done.; Combine taskbar buttons choices are Always, hide labels; When taskbar is full; and Never. For additional displays, the page names Combine taskbar buttons on other taskbars.; Lock the taskbar can be toggled On or Off. With multiple monitors, the page also mentions Lock all taskbars.; News and interests can be turned on or off under News and interests with Show news and interests on the taskbar.; Notification area settings include Select which icons appear on the taskbar and Turn system icons on or off.; Search is configured by right-clicking the taskbar and selecting Search. The choices are Hidden, Show search icon only, Show search box, Show search highlights, and Open on hover.; Show badges on taskbar buttons can be toggled On or Off; the page says this feature is turned on by default.; Taskbar height can be changed by unlocking the taskbar and dragging its border.; Taskbar hiding has separate options: Automatically hide the taskbar in desktop mode and Automatically hide the taskbar in tablet mode.; Taskbar location on screen can be set to Left, Top, Right, or Bottom when the taskbar is unlocked.; The Task view button can be hidden by right-clicking the taskbar and clearing Show Task View button.; The desktop Peek setting is Use Peek to preview the desktop when you move your mouse to the Show desktop button at the end of the taskbar.; Use small taskbar buttons can be toggled On or Off. The page notes that badges are not displayed on small taskbar buttons.

Preview/independent evidence (26H2-era):

- [Official Insider/Experience blog (T1, preview-scoped)](https://blogs.windows.com/windows-insider/2026/05/15/improving-windows-quality-making-taskbar-and-start-more-personal/) — applies to: Windows 11. No specific build or release version is stated..
  - Taskbar position (New option in Taskbar behaviors; the article does not give its exact UI label.): Top, bottom, left, or right. Moves the taskbar to the selected screen edge. Start, Search, and other flyouts appear relative to the taskbar location.
  - Taskbar icon alignment (Setting alongside the taskbar-position option; the article does not give its exact UI label.): Top-aligned or centered when the taskbar is on the left or right; left-aligned or centered when it is on the top or bottom. Sets icon alignment for the selected taskbar position.
  - Show smaller taskbar buttons (Setting): The article explicitly names the setting value “Always.”. When set to “Always,” icons and taskbar height become smaller, providing more vertical space. No restart or sign-out is required.
  - Requirement/rollout: Rolling out to Windows Insiders in the Experimental channel over the coming weeks; taskbar positions and the smaller-taskbar experience are described as rolling out starting today.
  - Requirement/rollout: No Copilot+ PC, region, or SKU requirement is stated.
- [Independent (T4)](https://www.windowscentral.com/microsoft/windows-11/windows-11-is-dangerously-close-to-letting-you-move-the-taskbar) — applies to: Builds 26100.9267/26200.9267. The page does not specify a Windows version for these taskbar changes..
  - Taskbar position (Choose where the taskbar appears): bottom, top, left, or right. The taskbar can appear at the selected screen edge. Tooltips, flyouts, and animations still come from the taskbar in the other positions; most customization settings, such as never combining taskbar icons, work with all locations.
  - Taskbar size (Choose the taskbar size): Small. When set to Small, both the icons and taskbar height become smaller, while access to Start, Search, and the system tray is maintained.
  - Taskbar behaviors (Settings section) — default: The section is expanded by default.. Expanded by default for improved discoverability.
  - Requirement/rollout: The article says the options are in the latest Windows 11 Insider Preview builds.
  - Requirement/rollout: Windows Insiders in the Release Preview Channel can start moving the taskbar immediately, according to the article.
  - Requirement/rollout: The article says the change should ship with the September Patch Tuesday update if testing goes well.
  - Requirement/rollout: The article says almost everything new in the builds is rolling out gradually; the normal rollout only includes “Additional quality improvements,” according to the change log.
  - Requirement/rollout: No Copilot+ PC, region, or SKU requirement is stated.


### WB.04.09 — Personalization > Fonts

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Settings > Personalization > Fonts`; `Settings > Personalization > Fonts > Related settings > Download fonts for all languages`

ms-settings URIs: `ms-settings:fonts`

Sources: [Official support article (T1)](https://support.microsoft.com/en-US/Windows/Experience/personalization/manage-fonts-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.04.09.o01 | Available fonts | section |  | — | Lists fonts available on the device. | — |
| WB.04.09.o02 | Search and filter | search feature |  | — | Type a font name in the search bar to locate a font. | — |
| WB.04.09.o03 | View a preview | preview feature |  | — | Shows a preview of a font, including different sizes and faces. | — |
| WB.04.09.o04 | Hide | action |  | — | Hides a font without uninstalling it. | — |
| WB.04.09.o05 | Uninstall | action |  | — | Uninstalls a font. The page says to select Options, then Uninstall when removing a font. | — |
| WB.04.09.o06 | Get more fonts in Microsoft Store | link |  | — | Opens Microsoft Store to get more fonts. | — |
| WB.04.09.o07 | Options | menu or button |  | — | Select this when removing a font, then select Uninstall. | — |
| WB.04.09.o08 | Download fonts for all languages | link or action |  | — | Under Related settings, select this to install supplemental fonts for other languages. | — |
| WB.04.09.o09 | OK | button |  | — | Select after choosing Download fonts for all languages. | — |



### WB.04.10 — Personalization > Device usage

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Settings > Personalization > Device usage`

ms-settings URIs: `ms-settings:deviceusage`

Sources: [Official support article (T1)](https://support.microsoft.com/en-US/Windows/Experience/personalization/personalize-your-windows-experience-with-device-usage-settings)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.04.10.o01 | Development | Toggle | On | — | When enabled, you might be offered an opportunity to use Dev Home to set up your PC for development. | — |
| WB.04.10.o02 | Gaming | Toggle | On | — | When enabled, you might be offered an Xbox Game Pass trial or receive suggestions on apps and sites based on what is popular with gamers. | — |
| WB.04.10.o03 | Family | Toggle | On | — | When enabled, you might see a notification to set up your Microsoft family group. | — |
| WB.04.10.o04 | Creativity | Toggle | On | — | When enabled, you might get app and site suggestions related to creativity. | — |
| WB.04.10.o05 | School | Toggle | On | — | When enabled, you might see suggestions to store your school files in OneDrive. | — |
| WB.04.10.o06 | Entertainment | Toggle | On | — | When enabled, you might see pinned app suggestions for entertainment apps and sites. | — |
| WB.04.10.o07 | Business | Toggle | On | — | When enabled, you might be offered a Microsoft 365 Business trial while using your PC. | — |


Preview/independent evidence (26H2-era):

- [OEM support (T4)](https://www.acer.com/ca-en/support/product-support/XF270W0/answers) — applies to: Windows 11; no specific build or version stated..
  - Gaming (Checkbox): On or off. Receive updates about new releases, Xbox Game Pass trials and app and site suggestions
  - Family (Checkbox): On or off. Set up a family group, edit safety settings and create individual profiles
  - Creativity (Checkbox): On or off. Receive various creativity suggestions for Videos, photos and other options
  - School (Checkbox): On or off. Note, Essay and collaboration support on OneDrive
  - Entertainment (Checkbox): On or off. Suggestions for apps, websites, videos and social media
  - Business (Checkbox): On or off. Tools for running your business and suggestions for Business 365
- [OEM support (T4)](https://www.acer.com/ca-en/support/product-support/Aspire_AG15-I31P) — applies to: Windows 11; no specific build or version stated..
  - Gaming (Checkbox): On or off (check or uncheck the box next to the option).. Receive updates about new releases, Xbox Game Pass trials and app and site suggestions.
  - Family (Checkbox): On or off (check or uncheck the box next to the option).. Set up a family group, edit safety settings and create individual profiles.
  - Creativity (Checkbox): On or off (check or uncheck the box next to the option).. Receive various creativity suggestions for Videos, photos and other options.
  - School (Checkbox): On or off (check or uncheck the box next to the option).. Note, Essay and collaboration support on OneDrive.
  - Entertainment (Checkbox): On or off (check or uncheck the box next to the option).. Suggestions for apps, websites, videos and social media.
  - Business (Checkbox): On or off (check or uncheck the box next to the option).. Tools for running your business and suggestions for Business 365.


## WB.05 — Apps


### WB.05.01 — Apps > Installed apps

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Apps > Installed apps`; `Settings > Apps > Installed apps > select the app > Advanced options`; `Start > Einstellungen > Apps > Installierte Apps`

ms-settings URIs: `ms-settings:appsfeatures`, `ms-settings:installed-apps` (not in MS Learn table — unverified)

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/uninstall-or-remove-apps-and-programs-in-windows-4b55f974-2cc6-2d2b-d092-5905080eaf98); [Community Q&A on Microsoft Learn (T5)](https://learn.microsoft.com/en-us/answers/questions/5790449/uninstalling-apps-win-11) (dated 2026-02-26); [Official support article (T1)](https://support.microsoft.com/de-de/windows/deinstallieren-oder-entfernen-von-apps-und-programmen-in-windows-4b55f974-2cc6-2d2b-d092-5905080eaf98)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.05.01.o01 | More | More | Uninstall | — | For the app you want to remove, select More > Uninstall. | Some apps can't be uninstalled from the Settings app right now. |
| WB.05.01.o02 | Uninstall | Uninstall |  | — | Starts uninstalling the selected app. | Some apps can't be uninstalled from the Settings app right now; the page directs users to Control Panel for help uninstalling those apps. |
| WB.05.01.o03 | Search box | Search box |  | — | Use it to find an app in Installed apps. | — |
| WB.05.01.o04 | Advanced options | Settings link/page for the selected app | Repair; Reset | — | Offers Repair or Reset. | Suggested if an app is misbehaving but cannot be removed; options are available only if shown. |
| WB.05.01.o05 | Mehr | Nicht angegeben |  | — | Wählen Sie bei der App, die Sie entfernen möchten, „Mehr“ aus. | Die App muss in „Installierte Apps“ gefunden werden. |
| WB.05.01.o06 | Deinstallieren | Nicht angegeben |  | — | Wählen Sie „Mehr“ > „Deinstallieren“, um die App zu entfernen. | Einige Apps können derzeit nicht über die Einstellungen-App deinstalliert werden. Verwenden Sie für diese Apps die Anweisungen zum Deinstallieren über die Systemsteuerung. |

- Confirmations / errors: After selecting Uninstall, follow the prompts.; Some built-in Windows apps cannot be uninstalled.


### WB.05.02 — Apps > Advanced app settings

Status: PARTIAL. Only community Q&A evidence; official documentation not found.

Navigation paths (as stated in sources): `Windows + I > Apps > Advanced app settings`; `Settings > Apps > Advanced app settings`; `Settings > System > Activation`; `Settings > Apps > Advanced apps settings`

ms-settings URIs: `ms-settings:advanced-apps` (not in MS Learn table — unverified), `ms-settings:appsfeatures-app`

Sources: [Community Q&A on Microsoft Learn (T5)](https://learn.microsoft.com/en-us/answers/questions/4270598/why-dont-i-have-the-app-and-features-in-settings) (dated 2022-10-20); [Community Q&A on Microsoft Learn (T5)](https://learn.microsoft.com/en-us/answers/questions/4005741/the-choose-where-to-get-my-apps-option-is-not-avai) (dated 2024-07-08); [Community Q&A on Microsoft Learn (T5)](https://learn.microsoft.com/en-us/answers/questions/2386421/i-am-trying-to-download-coreldraw-to-my-new-laptop?page=0) (dated 2023-08-25)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.05.02.o01 | Choose where to get apps | Selection control (type not specified) | Not stated | — | Change the selection to the desired one. | The answer says this setting is under Apps > Advanced app settings in the latest version of Windows 11. |
| WB.05.02.o02 | Share across devices | Not specified | Not stated | — | Not stated | A user lists this as an option shown on the page. |
| WB.05.02.o03 | App execution aliases | Not specified | Not stated | — | Not stated | A user lists this as an option shown on the page. |
| WB.05.02.o04 | Archive apps | Not specified | Not stated | — | Not stated | A user lists this as an option shown on the page. |
| WB.05.02.o05 | Related settings | Link or section; not specified | Not stated | — | Not stated | A user lists this on the page. |
| WB.05.02.o06 | Uninstall updates | Not specified | Not stated | — | Not stated | A user lists this under Related settings. |
| WB.05.02.o07 | How to install a program | Not specified | Not stated | — | Not stated | A user lists this under Related settings. |
| WB.05.02.o08 | Activation | Settings page | Not stated | — | The accepted answer says to open this page to check whether Windows is in S mode. | — |
| WB.05.02.o09 | Switching out of S mode in Windows | Link | Not stated | — | The accepted answer links to Microsoft Support guidance for switching out of S mode. | The accepted answer recommends switching out of S mode to install third-party apps. |
| WB.05.02.o10 | app executive aliases | Not stated |  | — | Listed by the user as an option they see; no behavior is described. | — |

- Confirmations / errors: A user reports that “Choose where to get apps” is not present and lists other items they see instead.; The accepted answer says the system is most likely in S mode and that Settings > System > Activation will show “Windows 11 Home in S mode” if that is the case.; The user says an app installer requires changing the setting for where apps can be obtained.; The user says the option is unavailable and that Advanced app settings displays only “Share across devices”, “Archive apps”, and “App execution aliases”.
- Keyboard shortcuts: Press Windows + I to open Settings.
- SKU / hardware / version conditions: The accepted answer identifies S mode as the likely reason the option is missing and says switching out of S mode is needed to install third-party apps.
- Windows 10 differences (legacy): The accepted answer says to switch out of S mode to Windows 10 Home; it does not explain a Windows 10 Settings-page difference.


### WB.05.03 — Apps > Default apps

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Settings > Apps > Default apps`

ms-settings URIs: `ms-settings:defaultapps`

Sources: [Official support article (T1)](https://support.microsoft.com/en-US/Windows/Apps/change-default-apps-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.05.03.o01 | Search bar | Search bar | File extensions or protocols, such as .txt | — | Search for a file extension or protocol, then select the corresponding result and choose an app to open that type. | — |
| WB.05.03.o02 | Set a default for a file type or link type | File type or link type association | Apps provided for the selected file type or link type | — | Select a file type or link type result, then choose an app to open files or links of that type. | — |
| WB.05.03.o03 | Set defaults for applications | Application selection | Applications listed in Settings, including Microsoft Edge | — | Select an app to see the file types or link types it can open; select a type to change its preferred app. | — |
| WB.05.03.o04 | Microsoft Edge | Application entry |  | — | Select Microsoft Edge under “Set defaults for applications” to configure its defaults. | To configure Microsoft Edge as the default browser. |
| WB.05.03.o05 | Make Microsoft Edge your default browser | Browser-default setting |  | — | Select “Set default” next to this setting. | Shown after selecting Microsoft Edge under “Set defaults for applications”. |
| WB.05.03.o06 | Set default | Button |  | — | Sets Microsoft Edge as the default browser. | Next to “Make Microsoft Edge your default browser” after selecting Microsoft Edge. |



### WB.05.04 — Apps > Offline maps

Status: PARTIAL. Official source confirms the page exists but documents few or none of its controls.

Navigation paths (as stated in sources): `System > Offline Maps`

ms-settings URIs: `ms-settings:maps`, `ms-settings:maps-downloadmaps`

Sources: [Official documentation (T1)](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-maps) (dated 2026-10-14)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.05.04.o01 | AllowOfflineMapsDownloadOverMeteredConnection | Device policy (int); no Settings control label or control type is stated. | 0 = Disabled. Force disable auto-update over metered connection.; 1 = Enabled. Force enable auto-update over metered connection.; 65535 (Default) = Not configured. User's choice. | 65535 (Not configured. User's choice.) | Allows the download and update of map data over metered connections. After the policy is applied, settings can be verified in System > Offline Maps. | Applies at device scope. The page lists Windows 10, version 1607 [10.0.14393] and later. |
| WB.05.04.o02 | EnableOfflineMapsAutoUpdate | Device policy (int); no Settings control label or control type is stated. | 0 = Disabled. Force off auto-update.; 1 = Enabled. Force on auto-update.; 65535 (Default) = Not configured. User's choice. | 65535 (Not configured. User's choice.) | The page says this policy “Disables the automatic download and update of map data.” It also says settings can be verified in System > Offline Maps. | Applies at device scope. The page lists Windows 10, version 1607 [10.0.14393] and later. Group Policy mapping: TurnOffAutoUpdate; Computer Configuration > Windows Components > Maps; registry value AutoDownloadAndUpdateMapData under Software\Policies\Microsoft\Windows\Maps; WinMaps.admx. |

- SKU / hardware / version conditions: Both policies list Pro, Enterprise, Education, and IoT Enterprise / IoT Enterprise LTSC editions, and Windows 10, version 1607 [10.0.14393] and later.
- Windows 10 differences (legacy): The page identifies the verification location as System > Offline Maps and lists Windows 10, version 1607 [10.0.14393] and later. It does not state the Windows 11 Settings path Apps > Offline maps or any Windows 11-specific differences.


### WB.05.06 — Apps > Apps for websites

Status: PARTIAL. Official source confirms the page exists but documents few or none of its controls.

Navigation paths (as stated in sources): `Settings > Apps > Apps for websites`; `Settings > Apps for Websites`

ms-settings URIs: `ms-settings:appsforwebsites`

Sources: [Official documentation (T1)](https://learn.microsoft.com/en-us/windows/apps/develop/launch/web-to-app-linking) (dated 2026-09-26)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.05.06.o01 | default link | Not stated | The user can choose which of the two apps is the default link if both apps are installed. | — | The user can change the default link in Settings > Apps for Websites. | Both apps must be installed. |



### WB.05.07 — Apps > Video playback

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Apps > Video playback`; `시작 > 설정 > 앱 > 비디오 재생`

ms-settings URIs: `ms-settings:videoplayback`

Sources: [Official support article (T1)](https://support.microsoft.com/en-US/Windows/Hardware/Display-Graphics/video-playback-settings-in-windows); [Official support article (T1)](https://support.microsoft.com/ko-kr/windows/video-playback-settings-in-windows-39ad5cbf-b2d1-2664-bd55-65028db05579)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.05.07.o01 | process video to enhance it |  |  | — | Have your PC process video to enhance it. | Depends on your PC’s hardware. |
| WB.05.07.o02 | play video at a lower resolution |  |  | — | Can help save network bandwidth. | — |
| WB.05.07.o03 | 배터리 절약 팁 | 링크 |  | — | 비디오를 재생할 때 배터리를 절약하는 방법에 대한 자세한 정보를 제공합니다. | — |

- Confirmations / errors: Different refresh rates on multiple monitors can cause video display problems, including playback stalling or repeatedly starting and stopping. This can occur with physically attached or wirelessly attached (Miracast) external monitors. Adjust the display refresh rate so all monitors use the same refresh rate.; 비디오 재생 문제가 발생하면 모든 모니터의 디스플레이 새로 고침 빈도를 같게 조정하라고 안내합니다.; 여러 모니터의 새로 고침 빈도가 다르면 비디오가 원활하게 재생되지 않거나 반복적으로 멈추고 다시 시작될 수 있습니다. 외부 모니터가 물리적으로 연결되었거나 Miracast로 무선 연결된 경우 모두 해당될 수 있습니다.
- SKU / hardware / version conditions: Video enhancement depends on your PC’s hardware.
- Windows 10 differences (legacy): Windows 10에도 적용된다고 표시되어 있지만, Windows 10 전용 차이는 언급되지 않았습니다.


### WB.05.08 — Apps > Startup

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Apps > Startup`

ms-settings URIs: `ms-settings:startupapps`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/Windows/experience/startup-boot/configure-startup-applications-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.05.08.o01 | Open Startup settings | Link |  | — | Opens Startup settings. | — |
| WB.05.08.o02 | On | Toggle | On / Off | — | Starts the app automatically when you sign in. Selecting the toggle can also show information about a missing app path. | For an application in the startup list. |
| WB.05.08.o03 | Off | Toggle | On / Off | — | Prevents the app from starting automatically. | For an application in the startup list. |
| WB.05.08.o04 | chevron | Expandable control |  | — | Opens advanced settings for the app. | For an application in the startup list. |

- Confirmations / errors: If an app is not listed in the startup list, its startup can be configured from File Explorer.; The Settings page may show “We couldn’t find this app” if the app is on a removable or network drive, its path does not exist, or it was uninstalled. Selecting the app’s On / Off toggle can show the registry location or file path containing the broken path. The page suggests reinstalling an uninstalled app; if the path points to a registry location, the Registry Editor can be used to find the broken path.; The page cautions that modifying the registry can have unintended consequences and recommends backing up the registry before making changes.


### WB.05.09 — Apps > Actions

Status: PARTIAL. Official source confirms the page exists but documents few or none of its controls.

Navigation paths (as stated in sources): `Apps`

Sources: [Official support article (T1)](https://support.microsoft.com/en-gb/windows/exploring-windows-settings-56bdfb5a-b809-4e65-9c90-4bab76745e19)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.05.09.o01 | Apps | Settings category/page |  | — | The page says Apps lets you view, install, uninstall, and update apps; choose default apps; and configure startup apps and actions. It does not describe an “Actions” page or identify any specific controls there. | — |


Preview/independent evidence (26H2-era):

- [Independent (T4)](https://gadgetsnurture.com/ai-actions-in-file-explorer-in-windows-11-enable-or-disable-it/) — applies to: Windows 11 25H2 update (2025 Update); the page says older versions will not show these options..
  - Describe image (System) (Toggle switch): On or off — default: Not stated. Listed as an app connected to AI Actions; the page does not specify its individual behavior.
  - Paint (Toggle switch): On or off — default: Not stated. Turning it off removes Paint AI options from the File Explorer context menu; Paint is described as providing background removal.
  - Photos (Toggle switch): On or off — default: Not stated. Turning it off removes Photos AI options from the File Explorer context menu, such as background blur or object removal.
  - Requirement/rollout: The page states that Windows 11 25H2 or later is required and that older versions will not show the options.
  - Requirement/rollout: It says AI Actions mainly work with JPG, JPEG, and PNG image files.
  - Requirement/rollout: It says some features, such as summarizing documents, may require a Microsoft 365 Copilot subscription.
  - Requirement/rollout: The page does not state a Copilot+ PC, Insider channel, region, or Windows SKU requirement.
- [Independent (T4)](https://www.elevenforum.com/t/enable-or-disable-recommended-actions-from-apps-in-windows-11.34473/) — applies to: Windows 11 build 26120.3576 (Dev), build 26100.3902 (RP), and build 26100.3915 (24H2)..
  - Actions (Per-app on/off setting): On or off — default: On (default). Manages which apps Windows can recommend actions from. Each app lists all recommended actions it is capable of.
  - Photos (Per-app on/off setting): On or off — default: Not stated specifically for Photos; the tutorial says available apps are on by default.. Turning off Photos turns off the available actions from the Photos app in Click To Do.
  - Requirement/rollout: Microsoft is beginning to introduce the Settings page in the listed Windows 11 builds.
  - Requirement/rollout: The page manages app recommendations for Copilot+ PCs.
  - Requirement/rollout: No region or SKU requirement is stated.


### WB.05.10 — Apps > App reset/repair (Advanced options)

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Apps > Installed apps`; `Start > Settings > Apps > Installed apps > More > Advanced options`

ms-settings URIs: `ms-settings:appsfeatures-app`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/topic/e90eefe4-d0a2-7c1b-dd59-949a9030f317)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.05.10.o01 | More | Menu/button |  | — | Select it to access Advanced options. | — |
| WB.05.10.o02 | Advanced options | Page |  | — | Opens the app’s advanced options page, where Repair or Reset may be available. | Some apps don't have this option. |
| WB.05.10.o03 | Repair | Button |  | — | Select it to repair the app. | Available only for some apps. |
| WB.05.10.o04 | Reset | Button |  | — | Select it if Repair isn't available or doesn't fix the problem. | Shown as the next option if Repair isn't available or doesn't fix the problem. |

- Windows 10 differences (legacy): The Settings path is Start > Settings > Apps > Apps & features. Select the app, then select Advanced options under its name. Repair and, if Repair is unavailable or does not fix the problem, Reset are described there.


## WB.06 — Accounts


### WB.06.01 — Accounts > Your info

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Settings > Accounts > Your info`

ms-settings URIs: `ms-settings:yourinfo`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/accounts-billing/manage/change-from-a-local-account-to-a-microsoft-account-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.06.01.o01 | Your info | shortcut/link |  | — | Opens the Your info page in Settings. | — |
| WB.06.01.o02 | Sign in with a Microsoft account instead | link |  | — | Starts the prompts to switch the local account to a Microsoft account. | Shown only if you're using a local account. |
| WB.06.01.o03 | Next | button |  | — | Continues the account-switching process. | — |
| WB.06.01.o04 | Sign out and finish | button |  | — | Signs out and finishes the account switch; sign back in afterward. | — |
| WB.06.01.o05 | Sign in with a local account instead | link |  | — | Starts the prompts to switch the Microsoft account to a local account. | Shown only if you're using a Microsoft account. |

- Restart / sign-out: Select “Sign out and finish,” then sign back in with the new account.


### WB.06.02 — Accounts > Email & accounts

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Settings > Accounts > Email & accounts`

ms-settings URIs: `ms-settings:emailandaccounts`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/security/identity-signin/manage-user-accounts-in-windows); [Official support article (T1)](https://support.microsoft.com/en-us/windows/manage-user-accounts-in-windows-104dc19f-6430-4b49-6a2b-e4dbd1dcdf32); [Official support article (T1)](https://support.microsoft.com/en-us/help/279783?nochrome=)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.06.02.o01 | Add an account | Button |  | — | Adds an account used by email, calendar, or contacts. | Under Accounts used by email, calendar, and contacts. |
| WB.06.02.o02 | Add a Microsoft account | Button |  | — | Adds an account for other apps. | For other apps. |
| WB.06.02.o03 | Add a work or school account | Button |  | — | Adds an account for other apps. | For other apps. |
| WB.06.02.o04 | Accounts used by email, calendar, and contacts | Section |  | — | For adding an account used by email, calendar, or contacts, select “Add an account” under this section. | — |

- Keyboard shortcuts: Email & accounts
- Windows 10 differences (legacy): It is no longer possible to add an Exchange account on Windows 10.; It's no longer possible to add an Exchange account on Windows 10.


### WB.06.03 — Accounts > Sign-in options

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Accounts > Sign-in options`

ms-settings URIs: `ms-settings:signinoptions`, `ms-settings:signinoptions-launchfaceenrollment`, `ms-settings:signinoptions-launchfingerprintenrollment`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/accounts-billing/security/sign-in-options-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.06.03.o01 | Set up Windows Hello | Sign-in setup option | Face recognition, a fingerprint, or a PIN instead of a password | — | Enables sign-in using Windows Hello. You can also improve facial recognition or remove it. | Availability of certain settings may vary based on the configuration of the device. |
| WB.06.03.o02 | physical security key | Sign-in method |  | — | Can be used to sign in to your account. | — |
| WB.06.03.o03 | Learn more | Link |  | — | Links to more information about signing in with a physical security key. | — |
| WB.06.03.o04 | Sign in with an external camera or fingerprint reader | Toggle | On or off | — | When turned on, disables Enhanced Sign-in Security, a Windows Hello feature that adds a layer of security to biometric data using specialized hardware and software components. | Available only on devices with compatible hardware. |
| WB.06.03.o05 | Only allow Windows Hello sign-in | Toggle | On or off | — | For Microsoft accounts on this device, turning it on disables the use of passwords. | Applies to Microsoft accounts on this device. |
| WB.06.03.o06 | Go passwordless with your Microsoft account | Link |  | — | Links to more information. | — |
| WB.06.03.o07 | Dynamic lock | Toggle | On or off | — | When enabled, uses Bluetooth to pair with a phone and automatically locks the device when the Bluetooth signal drops below a certain level. Windows locks within a minute after you move out of Bluetooth range. | Requires pairing with a phone over Bluetooth; take the phone with you when moving away from the device. |
| WB.06.03.o08 | Show account details on the sign-in screen | Toggle | On or off | — | When turned on, displays the user account's email address on the sign-in screen. | — |

- SKU / hardware / version conditions: Sign in with an external camera or fingerprint reader is available only on devices with compatible hardware.; The availability of certain settings may vary based on the configuration of the device.
- Windows 10 differences (legacy): The Windows 10 section does not mention the "Sign in with an external camera or fingerprint reader" setting.


### WB.06.03.01 — Accounts > Sign-in options > Dynamic lock

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Accounts > Sign-in options`

ms-settings URIs: `ms-settings:signinoptions-dynamiclock`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/accounts-billing/security/sign-in-options-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.06.03.01.o01 | Set up Windows Hello | Setup option | Face recognition, fingerprint, or PIN instead of a password | — | Enables sign-in using Windows Hello. You can also improve facial recognition or remove it. | — |
| WB.06.03.01.o02 | physical security key | Sign-in method; Learn more link |  | — | Can be used to sign in to your account. | — |
| WB.06.03.01.o03 | Sign in with an external camera or fingerprint reader. | Setting/toggle | On or off | — | When turned on, disables Enhanced Sign-in Security, a Windows Hello feature that adds a layer of security to biometric data using specialized hardware and software components. | Available only on devices with compatible hardware. This setting is listed in the Windows 11 section. |
| WB.06.03.01.o04 | Only allow Windows Hello sign-in | Setting/toggle | On or off | — | When turned on for Microsoft accounts on this device, disables the use of passwords for the Microsoft account. | — |
| WB.06.03.01.o05 | Dynamic lock | Setting/toggle | On or off | — | When turned on, uses Bluetooth to pair with a phone and automatically locks the device when the Bluetooth signal drops below a certain level. Windows locks within a minute after you move out of Bluetooth range. Take your phone with you when you move away from the device. | — |
| WB.06.03.01.o06 | Show account details on the sign-in screen | Setting/toggle | On or off | — | When turned on, displays the user account's email address on the sign-in screen. | — |
| WB.06.03.01.o07 | Learn more | Link |  | — | The page provides a Learn more link for the external-camera or fingerprint-reader setting. | — |
| WB.06.03.01.o08 | Go passwordless with your Microsoft account | Link |  | — | Further information about allowing only Windows Hello sign-in for Microsoft accounts. | — |

- SKU / hardware / version conditions: Sign in with an external camera or fingerprint reader is available only on devices with compatible hardware.; The availability of certain settings may vary based on the device configuration.
- Windows 10 differences (legacy): The Windows 10 section does not list the Windows 11 setting “Sign in with an external camera or fingerprint reader.”


### WB.06.04 — Accounts > Passkeys

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Settings > Accounts > Passkeys`; `Settings > Accounts > Passkeys > Advanced options`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/manage-your-passkeys-in-windows-6a70599a-25e1-4461-86be-d67d1023c69f)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.06.04.o01 | Passkeys | Shortcut to the Passkeys settings page |  | — | Opens the Passkeys page in Settings. | — |
| WB.06.04.o02 | Delete passkey | Button |  | — | Deletes a device-bound passkey stored on the Windows device. | Shown next to the passkey name. |
| WB.06.04.o03 | Advanced options | Sub-page |  | — | Provides advanced options for configuring passkey services, including integration with third-party passkey managers. | Located at Settings > Accounts > Passkeys. |
| WB.06.04.o04 | Passkey services available on your Windows device | Toggle(s) | On or off | — | Enables or disables available passkey services. | The feature may be disabled by your organization IT Admin. |
| WB.06.04.o05 | Save passkeys to this Windows device | Toggle | On or off | — | When turned on, provides the option to save or use passkeys stored locally on the device. | — |

- Keyboard shortcuts: Passkeys


### WB.06.05 — Accounts > Family

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `In the Windows Security app on your PC, select Family options.`; `Use the Family options shortcut.`

ms-settings URIs: `ms-settings:family-group`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/family-options-in-the-windows-security-app-5e8b9fd8-8372-dba0-eba2-46da8e407026)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.06.05.o01 | Family options | Windows Security app page/shortcut |  | — | Opens the Family options page in Windows Security. | Available on Windows 11 and Windows 10. |
| WB.06.05.o02 | View family settings | Link or command |  | — | Goes to the Family Safety website to adjust family settings or set up a family. | — |
| WB.06.05.o03 | Install Microsoft Family Safety | Link | Install on a mobile device, or on a PC from the Microsoft Store. | — | Offers installation of the Microsoft Family Safety app. | — |
| WB.06.05.o04 | Getting started with Microsoft Family Safety | Link |  | — | Links to more information about getting started with Microsoft Family Safety. | — |

- Keyboard shortcuts: Family options


### WB.06.06 — Accounts > Windows backup

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Settings > Accounts > Windows backup`; `Settings > Accounts > Your info`

ms-settings URIs: `ms-settings:backup`, `ms-settings:sync`

Sources: [Official support article (T1)](https://support.microsoft.com/help/4026102/windows-recovery-environment); [Official support article (T1)](https://support.microsoft.com/en-us/windows/experience/backup-recovery/back-up-and-restore-with-windows-backup); [Official support article (T1)](https://support.microsoft.com/help/17128)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.06.06.o01 | Remember my apps | Switch |  | — | Backs up the list of installed apps. | Windows backup must be turned on. |
| WB.06.06.o02 | Remember my preferences | Not specified | Accessibility; Accounts and passwords; Personalization; Language preferences; Other Windows settings | — | Identifies the preferences categories backed up by Windows. | Windows backup must be turned on. |
| WB.06.06.o03 | Accounts and passwords | Checkbox |  | — | Covers Wi-Fi profiles (only WPA profiles) and web credentials. | Listed under Remember my preferences. |
| WB.06.06.o04 | Manage sync settings | Button/link under “OneDrive folder synching” | Desktop; Documents; Pictures; Videos; Music | — | Opens the folder-sync settings, where you can toggle which folders to back up to OneDrive. | — |
| WB.06.06.o05 | Desktop; Documents; Pictures; Videos; Music | Individual toggles in the folder-sync settings | On or off | — | Controls whether each listed user folder is backed up to OneDrive. | Reached through “Manage sync settings” under “OneDrive folder synching”. |
| WB.06.06.o06 | Installed apps | Backup setting listed on the page |  | — | Listed as an available Windows setting to back up. | — |
| WB.06.06.o07 | Personalization | Backup setting listed on the page | Wallpaper, color, theme, start layout, etc. | — | Listed as an available Windows setting to back up. | Requires OneDrive sign-in and available storage; preferences will not be backed up if OneDrive storage is insufficient. |
| WB.06.06.o08 | Other Windows settings | Checkbox under “Remember my preferences” | Checked or unchecked | — | When turned on, Windows also syncs some device settings (such as printer and mouse options), File Explorer settings, and notification preferences. | Available for a work or school account if allowed by the organization. |
| WB.06.06.o09 | Clear stored settings | Selection on the Microsoft Account Devices page |  | — | Deletes data previously backed up to the Microsoft account. | Use if backup has been turned off and you want to delete previously backed-up data; find it in the “Cloud synced settings” section at the bottom of the Microsoft Account Devices page. |

- Confirmations / errors: If settings have been backed up in the past, some by default, they show as “Backed up” and their toggle is unavailable. The page does not specify which settings are backed up by default.; The backup status can be checked in Settings > Accounts > Windows backup.; The page does not state default values for the Windows 11 Settings controls.; The page says OneDrive backup errors can occur if you are not signed in to OneDrive, have insufficient storage, or another OneDrive issue needs to be resolved.
- Keyboard shortcuts: Settings > Accounts > Windows backup; Settings > Accounts > Your info
- SKU / hardware / version conditions: The Windows Backup app is described as focused on consumer devices using a personal Microsoft account; work or school Microsoft accounts do not work with the app.; Windows Backup is described as focused on consumer devices using a personal Microsoft account; work or school Microsoft accounts will not work with the Windows Backup app.; “Remember my apps” is not available for work or school accounts. “Other Windows settings” may be available for a work or school account if the organization allows it.
- Windows 10 differences (legacy): The Windows 10 app-backup toggle is labeled “Get quick access to apps on Windows device”; the Windows 11 Settings instructions label it “Remember my apps.”; The Windows 10 app-backup toggle is labeled “Get quick access to apps on Windows device”; the Windows 11 label is “Remember my apps”.; The Windows 10 apps switch is labeled “Get access to apps on Windows device.” Its sync settings include Accessibility, Wi-Fi networks and passwords, Personalization, Language preferences, and Other Windows settings.; The Windows 10 configuration instructions use “Back up files” to change which folders synchronize to OneDrive; the Windows 11 instructions say to use “Manage sync settings” under “OneDrive folder synching”.; The Windows 10 credentials switch is labeled “Wi-Fi networks and passwords.”; The Windows 10 instructions describe the items under “Remember my preferences” as toggles; the Windows 11 instructions describe them as checkboxes.; The Windows 10 instructions say to select “Back up files” and modify which folders synchronize to OneDrive; the Windows 11 instructions instead say to select “Manage sync settings” under “OneDrive folder synching.”; The Windows 10 text says the OneDrive Backup status can be checked from the Settings homepage. The Windows 11 text says to check status in Settings > Accounts > Windows backup.; Windows 10 uses Settings > Accounts > Sync your settings rather than Accounts > Windows backup.


### WB.06.07 — Accounts > Other users

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Settings > Accounts > Other user`

ms-settings URIs: `ms-settings:otherusers`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/security/identity-signin/manage-user-accounts-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.06.07.o01 | Other users | Settings page/section |  | — | Shows the area where you can add, remove, or change another user's account. | — |
| WB.06.07.o02 | Add other user | Section |  | — | Contains the option to add an account. | — |
| WB.06.07.o03 | Add account | Button |  | — | Starts adding a user account and prompts for the person's sign-in information. | — |
| WB.06.07.o04 | I don't have this person's sign-in information | Option |  | — | Lets you sign up for a new email address and create a new Microsoft account, or choose to create a local account. | Shown during the add-account flow. |
| WB.06.07.o05 | Sign up for a new email address | Option |  | — | Starts the process of creating a new Microsoft account. | For a person who does not already have a Microsoft account. |
| WB.06.07.o06 | Add a user without a Microsoft account | Option |  | — | Creates a local account. | Available in the add-account flow. |
| WB.06.07.o07 | Account and data | Section label |  | — | Shows the Remove option for the selected account. | After selecting the account's flyout. |
| WB.06.07.o08 | Remove | Button |  | — | Removes the account's sign-in information and data from the device. | Next to Account and data for the selected account. |
| WB.06.07.o09 | Account options | Section label |  | — | Shows the Change account type option for the selected account. | After selecting the account's flyout. |
| WB.06.07.o10 | Change account type | Button |  | — | Opens the account-type selection control. | Next to Account options for the selected account. |
| WB.06.07.o11 | Account type | Dropdown | Account types; specific choices are not listed. | — | Select an account type, then select OK. | After selecting Change account type. |
| WB.06.07.o12 | OK | Button |  | — | Applies the selected account type. | After choosing an account type. |

- Confirmations / errors: Removing an account does not delete the person's Microsoft account; it removes their sign-in information and data from the device.; The page recommends limiting the number of administrators because administrators have complete control over the system; it recommends standard accounts for day-to-day activities.


### WB.06.08 — Accounts > Access work or school

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Settings > Accounts > Access work or school`

ms-settings URIs: `ms-settings:workplace`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/4fac6fd5-74c0-9737-69b8-6e77e00422dc)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.06.08.o01 | Add a work or school account | Button |  | — | Starts connecting a work or school account. | The organization must support personal devices or BYOD scenarios. |
| WB.06.08.o02 | Connect | Button |  | — | Begins adding a work or school account. | — |
| WB.06.08.o03 | account type | Selection control; exact control type not stated |  | — | Select the account type before selecting Add. | — |
| WB.06.08.o04 | Add | Button |  | — | Adds the account after account information and account type are entered or selected. | — |
| WB.06.08.o05 | Disconnect | Dropdown menu option |  | — | Removes the work or school account’s sign-in information and data from the device; it does not delete the account. | Accessed from the dropdown next to the work or school account. |
| WB.06.08.o06 | Yes | Confirmation button |  | — | Confirms disconnecting the account. | Shown after selecting Disconnect. |

- Confirmations / errors: After selecting Disconnect, select Yes to confirm.
- SKU / hardware / version conditions: To connect a work or school account, the organization must support personal devices or bring your own device (BYOD) scenarios.


## WB.07 — Time & language


### WB.07.01 — Time & language > Date & time

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Time & language > Date & time`; `Open Time & language > Date & time`

ms-settings URIs: `ms-settings:dateandtime`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/experience/personalization/set-time-date-and-time-zone-settings-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.07.01.o01 | Set time automatically | Toggle | On; Off | — | When On, the date and time are set automatically. To set them manually, turn this Off. | — |
| WB.07.01.o02 | Set the date and time | Change button |  | — | Opens the Change date and time window for manually setting the date and time. | Set time automatically must be Off. |
| WB.07.01.o03 | Date | Field in the Change date and time window |  | — | Manually set the date. | Set time automatically must be Off. |
| WB.07.01.o04 | Time | Field in the Change date and time window |  | — | Manually set the time. | Set time automatically must be Off. |
| WB.07.01.o05 | Change | Button in the Change date and time window |  | — | Applies the manually set date and time. | — |
| WB.07.01.o06 | Set time zone automatically | Toggle | On; Off | — | When On, the time zone is set automatically. To select a time zone manually, turn this Off. | — |
| WB.07.01.o07 | Time zone | Drop-down menu | Desired time zone | — | Select the desired time zone. | Set time zone automatically must be Off. |
| WB.07.01.o08 | Adjust for daylight saving time automatically | Toggle | On; Off | — | When On, daylight saving time is adjusted automatically. When Off, set the time manually if needed. | Applies when Set time zone automatically is Off. When Set time zone automatically is On, daylight saving time is set automatically. |
| WB.07.01.o09 | Show time and date in the System tray | Toggle | On; Off | On; the date and time are shown in the system tray by default. | On adds the date and time to the system tray; Off removes them. | — |
| WB.07.01.o10 | Show seconds in system tray clock (uses more power) | Option |  | The system tray time normally doesn't show seconds. | Shows seconds in the system tray clock. | — |
| WB.07.01.o11 | Open Time & language > Date & time | Link |  | — | Opens Time & language > Date & time. | — |

- Windows 10 differences (legacy): For manual date and time setting, Windows 10 says to select the Change button under Set the date and time manually; the Windows 11 section says to select the Change button next to Set the date and time.; The Windows 10 section does not include the system tray date-and-time settings.


### WB.07.02 — Time & language > Language & region

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Time & language > Language & region`

ms-settings URIs: `ms-settings:regionlanguage`, `ms-settings:regionformatting`, `ms-settings:keyboard`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/manage-the-input-and-display-language-settings-in-windows-12a10cb4-8626-9b77-0ccb-5013e0c7c7a2?nochrome=)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.07.02.o01 | Open Language & region | Link |  | — | Opens the Language & region page. | — |
| WB.07.02.o02 | Language | Section | Currently installed languages | — | Displays currently installed languages. | — |
| WB.07.02.o03 | Add a language | Button |  | — | Opens the language installation flow. | Language packs are downloaded via the Internet; the device must be connected to the Internet. |
| WB.07.02.o04 | Type a language name | Text box | Language names, or scroll through the available languages | — | Finds a language to install. | Selecting a language opens the Install language features window. |
| WB.07.02.o05 | Language pack | Optional language feature selection | Selected or not selected | The article says to make sure it is selected. | Installs the language pack. | Shown in the Install language features window under Optional language features. |
| WB.07.02.o06 | Set as my Windows display language | Language preference selection | Selected or not selected | — | Sets the language being installed as the Windows display language. | Shown in the Install language features window under Language preferences. Selecting it requires signing out to apply the display language. |
| WB.07.02.o07 | Install | Button |  | — | Installs the selected language and chosen features. | In the Install language features window. |
| WB.07.02.o08 | Windows display language | Drop-down menu | Installed Windows display languages | — | Sets the language used by the Windows user interface and Windows features such as Settings and File Explorer. The selected language moves to the top of Preferred languages. | The desired language must already be installed. Applying a change requires signing out and signing back in. |
| WB.07.02.o09 | Sign out | Button |  | — | Signs out so the new Windows display language can be applied. | Displayed after selecting or installing a language as the Windows display language. If the button is absent, the article says to use Start > Shutdown or sign out > Sign out. |
| WB.07.02.o10 | Preferred languages | Ordered language list | Installed languages | — | The first supported language determines the language used by Microsoft Store apps and websites. Languages can be dragged to reorder them; moving a language to the top does not change the selected Windows display language. | — |
| WB.07.02.o11 | Language options | Ellipses menu action |  | — | Opens the Options window for a language, which includes Language Features and Keyboards. | Select the ellipses for a language in the Language section. |
| WB.07.02.o12 | Download | Button | Text-to-speech; Basic Speech recognition; Enhanced speech recognition | — | Downloads the selected speech feature. | Shown next to available speech features under Language Features in the Options window. The language must be set as the display language, and the device must be connected to the Internet. Not all languages offer all these features. |
| WB.07.02.o13 | Add a keyboard | Button | Keyboard layouts available in the pop-up menu | — | Adds the selected keyboard layout under Keyboards in the Options window. | Use the Options window for a language. If the desired layout is not listed, the article says to check the language that matches the layout; that language may need to be installed. |
| WB.07.02.o14 | Remove | Ellipses menu action / button |  | — | Removes a language pack from the Language section, or removes a keyboard layout from the Keyboards section in the Options window. | A language cannot be removed while it is the current Windows display language. A keyboard layout cannot be removed if it is the only installed layout for the current Windows display language. |

- Confirmations / errors: A single-language Windows edition may show “Only one language pack allowed” or “Your Windows license supports only one display language” when adding a language pack.; Applying a newly selected Windows display language requires signing out and signing back in.; Changing the Windows display language might also change the keyboard layout. The article advises checking the input language icon at sign-in and notes that changing the keyboard layout does not change the Windows display language.; If Remove is unavailable for a keyboard layout, it is the only installed keyboard layout for the current Windows display language; add another layout first.; If Remove is unavailable for a language, it is the current Windows display language; switch to another display language first.; Language installation displays a message that the language is still installing and may take a few minutes; download and installation progress is shown in the language list.; Not all languages have speech features; unavailable speech features are not offered as options.; Some languages are not fully localized, so some Windows elements may remain in the first fully localized language in Preferred languages.
- Restart / sign-out: If the Sign out button is not displayed, the article says to right-click Start > Shutdown or sign out > Sign out.; Sign out of Windows and sign back in to apply a newly selected Windows display language.
- Keyboard shortcuts: Windows logo key + Spacebar: select a keyboard layout; each press of Spacebar while holding the Windows logo key selects another layout.
- SKU / hardware / version conditions: A Windows edition may allow only one language pack or one display language, as indicated by the messages “Only one language pack allowed” or “Your Windows license supports only one display language”.
- Windows 10 differences (legacy): For Windows 10, the article describes showing the taskbar input language icon through Start > Settings > Personalization > Taskbar > Turn system icons on or off > Input Indicator. It also gives the legacy language bar path as Start > Settings > Devices > Typing > Advanced keyboard settings.; The Windows 10 language installation option is labeled “Install language pack”; the article says to select it under Optional language features.; The Windows 10 navigation path is Start > Settings > Time & language > Language, rather than Language & region.; Windows 10 language options show speech downloads under Speech, and keyboard layouts under Keyboards.; Windows 10 uses the Language window and Preferred language list. Adding a language uses Add a language; language options are opened by selecting a language and then Options.


### WB.07.03 — Time & language > Typing

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Time & language > Typing`; `Settings > Time & Language > Typing`

ms-settings URIs: `ms-settings:typing`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/accessibility/windows/enable-text-suggestions-in-windows); [Community Q&A on Microsoft Learn (T5)](https://learn.microsoft.com/en-us/answers/questions/3901671/i-am-getting-suggestions-even-in-this-text-box-for) (dated 2025-02-28); [Official support article (T1)](https://support.microsoft.com/en-gb/windows/enable-text-suggestions-in-windows-0bf313ca-c992-4173-aa5f-8341d3953498) (dated 2022-03-16)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.07.03.o01 | Show text suggestions when typing on the physical keyboard | Switch | On or off | — | When turned on, suggested words appear as you type on a physical keyboard. Select a suggestion with a mouse or touch, or use the arrow keys and Enter. Click outside the suggestions window or press Esc to dismiss suggestions without inserting one. | Applies when typing on a physical keyboard. |
| WB.07.03.o02 | Multilingual text suggestions | Switch | On or off | — | Enables text prediction in several languages. | Turn on if you want to use text prediction in several languages. |
| WB.07.03.o03 | Show text suggestions as I type | Toggle | On/Off | — | Turn off to stop text suggestions while typing on a hardware keyboard. | For hardware keyboards. |
| WB.07.03.o04 | Autocorrect misspelled words | Toggle | On/Off | — | Can be turned off. | For hardware keyboards; offered as an additional preference. |
| WB.07.03.o05 | Highlight misspelled words | Toggle | On/Off | — | Can be turned off. | For hardware keyboards; offered as an additional preference. |
| WB.07.03.o06 | Show text suggestions when typing on the touch keyboard | Toggle | On/Off | — | Turn off to disable touch-keyboard suggestions. | For the touch keyboard, such as when using a touchscreen or tablet. |
| WB.07.03.o07 | Accessibility support for Windows | link |  | — | Listed under “See also.” | — |

- Confirmations / errors: A follow-up answer says the user needed to restart for the settings changes to take effect.
- Restart / sign-out: The follow-up answer says a restart was needed for the changes to take effect.
- Keyboard shortcuts: Esc dismisses suggestions without inserting one.; Press Esc to dismiss suggestions without inserting one. Clicking outside the suggestions window also dismisses them.; To select a suggestion: press Up arrow, use Left and Right arrows to navigate, then press Enter.; Up arrow, then Left and Right arrow keys to navigate suggestions; press Enter to select a suggestion.; Win + I opens Settings.; Windows logo key+I opens Windows Settings.
- SKU / hardware / version conditions: Text suggestions from the first switch apply to typing on a physical keyboard.; The answer distinguishes hardware keyboard settings from touch keyboard settings.; Touch selection is available if the PC supports it.
- Windows 10 differences (legacy): The Windows 10 instructions use Windows Settings > Devices > Typing.; The Windows 10 multilingual setting is labeled “Show text suggestions based on the recognized languages you’re typing in.”; The Windows 10 path shown is Settings > Devices > Typing, rather than Time & language > Typing.; The multilingual setting is labeled “Show text suggestions based on the recognized languages you're typing in.”; Under Hardware keyboard, the setting is labeled “Show text suggestions as I type.”; Under Typing, the setting is labeled “Show text suggestions as I type on the software keyboard.”; Windows 10 also lists “Show text suggestions as I type on the software keyboard” under “Typing.”; Windows 10 labels the physical-keyboard setting “Show text suggestions as I type” under “Hardware keyboard.”


### WB.07.04 — Time & language > Speech

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Time & Language > Speech`; `Start > Settings > Time & language > Speech`

ms-settings URIs: `ms-settings:speech`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/education/learning-accelerators/download-languages-and-voices-for-immersive-reader-read-mode-and-read-aloud); [Official support article (T1)](https://support.microsoft.com/en-us/topic/4c83a8d8-7486-42f7-8e46-2b0fdf753130?nochrome=); [Official support article (T1)](https://support.microsoft.com/en-us/windows/use-voice-recognition-in-windows-83ff75bd-63eb-0b6c-18d4-6fae94050571)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.07.04.o01 | Speech language | Dropdown | Choose your desired language. | — | Selects the speech language. | Text-to-speech must be available in the language. |
| WB.07.04.o02 | Recognize non-native accents for this language | Not stated |  | — | Detects and translates varying accents within the language. | Listed under Speech options. |
| WB.07.04.o03 | Voices | Voice settings | Default voice, speed, and voice preview. | — | Change the default voice and its speed, and preview the voice. | Listed under Speech options. |
| WB.07.04.o04 | Get started | Button |  | — | Walks through the Cortana mic setup. | In the Microphone section. |

- Confirmations / errors: If the wizard detects microphone issues, it lists them in the wizard dialog box. You can select options in the dialog box to specify an issue and help the wizard solve it; the page does not name those options.
- Windows 10 differences (legacy): The Windows 10 instructions also say to select the Start button before Settings. Both versions say to select Get started under Microphone.; The Windows 10 instructions use the path Start > Settings > Time & Language > Speech; the Windows 11 instructions capitalize “Language,” while the Windows 11 path is shown as “Time & language.”


## WB.08 — Gaming


### WB.08.01 — Gaming > Game Bar

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Settings > Gaming > Xbox Game Bar (on some versions it’s just Game Bar)`; `Settings > Gaming > Game Bar`

ms-settings URIs: `ms-settings:gaming-gamebar`

Sources: [Community Q&A on Microsoft Learn (T5)](https://learn.microsoft.com/en-us/answers/a/12998230) (dated 2026-10-13); [Official support article (T1)](https://support.microsoft.com/vi-vn/accessibility/windows/use-a-screen-reader-to-record-your-screen-with-xbox-game-bar) (dated 2026-06-24); [Community Q&A on Microsoft Learn (T5)](https://learn.microsoft.com/en-us/answers/questions/5643066/the-new-dec-2025-game-bar-keeps-stealing-focus) (dated 2025-12-02)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.08.01.o01 | Xbox Game Bar (on some versions: Game Bar) | On/off control; exact control type not stated | Not stated | — | Turn it on. Pressing Win + G opens Game Bar; if it opens, Game Bar itself works. | — |
| WB.08.01.o02 | Allow your controller to open Game Bar | Checkbox or toggle; the page says it should be “ticked” | Not stated | — | Make sure it is ticked. | Relevant when using the controller to open Game Bar. |
| WB.08.01.o03 | Open Xbox Game Bar using this button on a controller, Xbox button | toggle switch | The screen reader announces “off”; the article says to press Space to enable it. | — | Press Space to turn on Xbox Game Bar. | — |

- Keyboard shortcuts: Press Space to enable the toggle.; Win + G; Windows+I opens Windows Settings. Type “xbox,” press Down Arrow until “Enable Xbox Game Bar” is announced, then press Enter.
- Windows 10 differences (legacy): No Windows 10-only differences about Gaming > Game Bar are stated.


### WB.08.02 — Gaming > Captures

Status: PARTIAL. Only community Q&A evidence; official documentation not found.

Navigation paths (as stated in sources): `Settings > Gaming > Captures`; `Settings > Games > Capture`

ms-settings URIs: `ms-settings:gaming-gamedvr`

Sources: [Community Q&A on Microsoft Learn (T5)](https://learn.microsoft.com/en-ca/answers/questions/5766126/windows-alt-r-is-not-working-to-record-screen) (dated 2026-02-07); [Community Q&A on Microsoft Learn (T5)](https://learn.microsoft.com/en-us/answers/questions/3893614/is-there-a-way-to-see-a-recording-that-did-not-ref) (dated 2025-02-25)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.08.02.o01 | Recording options (video/audio) | Not stated | Not stated | Not stated | Review the recording options. | Not stated |
| WB.08.02.o02 | Save location | Not stated | Not stated | Not stated | Ensure the save location is accessible. | Not stated |
| WB.08.02.o03 | Max recording length | Not stated | Not stated | — | Set it up to a longer time. | Not stated |
| WB.08.02.o04 | Record what happened | Not stated | Not stated | — | Disable the setting. | Not stated |



### WB.08.03 — Gaming > Game Mode

Status: PARTIAL. Official source confirms the page exists but documents few or none of its controls.

Navigation paths (as stated in sources): `Settings > Gaming > Game Mode`; `Start > Settings > Gaming > Game Mode`; `Gaming > Game Mode`; `Gaming -> Game Mode -> Game Mode`

ms-settings URIs: `ms-settings:gaming-gamemode`

Sources: [Official support article (T1)](https://support.microsoft.com/id-id/windows/perkenalkan-windows-11-gaming-2e537161-a578-4ebd-b399-366c855023e1) (dated 2022-06-20); [Official support article (T1)](https://support.microsoft.com/en-us/help/4028293/windows-using-game-mode-on-your-pc); [Official documentation (T1)](https://learn.microsoft.com/en-us/windows/apps/develop/settings/settings-windows-11) (dated 2026-04-14)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.08.03.o01 | Game Mode | Setting |  | On by default | The page recommends ensuring it is enabled for the best gaming performance. | — |

- Confirmations / errors: Did this resolve the issue? — Yes; No


## WB.09 — Accessibility


### WB.09.01 — Accessibility > Text size

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Accessibility > Text size`; `Windows 11: Start > Settings > Accessibility > Text size`

ms-settings URIs: `ms-settings:easeofaccess-display`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/accessibility/windows/make-text-and-apps-bigger); [Official support article (T1)](https://support.microsoft.com/en-us/accessibility/windows/change-the-size-of-text-in-windows); [Official support article (T1)](https://support.microsoft.com/en-us/microsoft-edge/change-the-size-of-text-in-windows-30e8aed9-2eee-9326-5256-2e5d58b9f176)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.09.01.o01 | Text size | Slider |  | — | Drag the slider to the right to increase the size of the sample text. Selecting Apply scales up the size of all text. | — |
| WB.09.01.o02 | Apply | Button |  | — | Applies the selected text size. | — |
| WB.09.01.o03 | Text size preview | Slider |  | — | Adjust it to make just the text on your screen larger. | — |
| WB.09.01.o04 | Display | Button |  | — | Opens Display settings, where you can choose an option from the drop-down menu next to Scale to make everything larger, including images and apps. | Listed under related setting. |
| WB.09.01.o05 | Scale | Drop-down menu | Options are not specified on this page. | — | Choose an option to make everything larger, including images and apps. | Accessed through the Display button under related setting. |
| WB.09.01.o06 | Open Display settings | Link |  | — | Opens Display settings. | — |

- Keyboard shortcuts: Windows logo key+U
- Windows 10 differences (legacy): The drop-down menu is under “Make everything bigger” and changes the size of everything, including images and apps.; The slider is labeled “Make text bigger” and makes just the text larger.; Windows 10 path: Start > Settings > Ease of Access > Display.; Windows 10 uses Ease of Access settings: press Windows key+U, then under Make text bigger on the Display tab, drag the slider and select Apply.; Windows 10 uses Start > Settings > Ease of Access > Display. Its text-size slider is labeled “Make text bigger,” and its drop-down menu is labeled “Make everything bigger.”


### WB.09.02 — Accessibility > Visual effects

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Accessibility > Visual effects`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/accessibility/windows/make-it-easier-to-focus-on-tasks)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.09.02.o01 | Always show scrollbars | Toggle |  | — | Turn off to automatically hide scrollbars in Windows. | — |
| WB.09.02.o02 | Transparency effects | Toggle |  | — | Turn off to make window backgrounds more opaque. | — |
| WB.09.02.o03 | Animation effects | Toggle |  | — | Turn off to prevent Windows from showing animations. | — |
| WB.09.02.o04 | Dismiss notifications after this amount of time | Dropdown menu |  | — | Select an option to define how long notifications are shown. | — |

- Windows 10 differences (legacy): The Windows 10 section uses Start > Settings > Ease of Access > Display instead of the Windows 11 Visual effects page.; Windows 10 names the related toggles Show animations in Windows, Show transparency in Windows, and Automatically hide scroll bars in Windows. It also lists a Show desktop background image toggle.


### WB.09.03 — Accessibility > Mouse pointer & touch

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Accessibility > Mouse pointer and touch`; `Windows logo key+U > Mouse pointer and touch`; `Windows 11: Start > Settings > Accessibility > Mouse pointer & touch`

ms-settings URIs: `ms-settings:easeofaccess-mousepointer`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/accessibility/windows/make-your-mouse-keyboard-and-other-input-devices-easier-to-use); [Official support article (T1)](https://support.microsoft.com/en-us/accessibility/office-accessibility/set-up-your-device-to-work-with-accessibility-in-microsoft-365); [Official support article (T1)](https://support.microsoft.com/en-us/accessibility/windows/make-windows-easier-to-see)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.09.03.o01 | Size | Slider |  | — | Adjusts the mouse pointer size. | Under Mouse pointer. |
| WB.09.03.o02 | Mouse pointer style | Options | White, black, inverted, or any of the bright recommended colors. | — | Changes the mouse pointer color. | — |
| WB.09.03.o03 | Choose another color | Button |  | — | Lets you customize the pointer color. | Under Mouse pointer style. |
| WB.09.03.o04 | Mouse pointer and touch | Settings page/menu item |  | — | Opens the settings described for changing mouse pointer size and color. | Windows 11 |
| WB.09.03.o05 | Make your mouse, keyboard, and other input devices easier to use | Link |  | — | Opens more information. | — |
| WB.09.03.o06 | color and size of the mouse pointer | Options on the Mouse pointer & touch page; the page does not name specific controls | Not stated | — | Change the mouse pointer’s color and size to make it more visible. | — |
| WB.09.03.o07 | Mouse pointer trails | Toggle | On or off | — | When turned on, shows where the mouse is moving on the screen. | — |
| WB.09.03.o08 | Touch indicator | Toggle | On or off | — | Shows visual feedback when you touch the screen. | — |
| WB.09.03.o09 | Make the circle darker and larger | Checkbox | Selected or not selected | — | Makes the touch indicator circle more visible. | Expand the Touch indicator settings. |
| WB.09.03.o10 | Change text cursor & mouse pointer | Link | Not stated | — | The page presents this link after the mouse and touch guidance. | — |

- Keyboard shortcuts: Windows logo + U; Windows logo key+U opens the Accessibility menu.
- Windows 10 differences (legacy): The Windows 10 path is Start > Settings > Ease of Access > Mouse pointer.; Windows 10 calls the size control “Change pointer size” and says to choose a color from white, black, inverted, or the bright suggested colors; “Pick a customer pointer color” selects a color from a color wheel.; Windows 10 uses Start > Settings > Devices > Mouse > Additional mouse options > Pointer Options > Display pointer trails for pointer trails.; Windows 10 uses Start > Settings > Ease of Access > Mouse pointer > Show visual feedback around the touch points when I touch the screen for touch feedback. It names the checkbox Make visual feedback for touch points darker and larger.; Windows 10 uses Start > Settings > Ease of Access > Mouse pointer for mouse pointer color and size options.; Windows 10 uses the path Ease of Access > Mouse pointer. It has a Change pointer size slider and a Change pointer color section; the article does not give the Windows 11 labels for these settings.


### WB.09.04 — Accessibility > Text cursor

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Accessibility > Text cursor`

ms-settings URIs: `ms-settings:easeofaccess-cursor`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/make-your-pc-easier-to-use-1c313f8f-8bc3-4e1c-c1c8-efe173a94845)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.09.04.o01 | Text cursor thickness | Slider |  | — | Adjusts the thickness of the text cursor; the preview shows how it looks. | — |
| WB.09.04.o02 | Text cursor indicator | Switch | On or off | — | Turns the text cursor indicator on or off. | — |
| WB.09.04.o03 | Size | Slider |  | — | Adjusts the text cursor indicator size; the preview shows how it looks. | Under Text cursor indicator. |
| WB.09.04.o04 | Recommended colors | Menu | Recommended colors | — | Select a color for the text cursor indicator. | Under Text cursor indicator. |
| WB.09.04.o05 | Choose another color | Button | Custom color | — | Lets you choose a custom color for the text cursor indicator. | Under Text cursor indicator. |

- Keyboard shortcuts: Windows logo + U
- Windows 10 differences (legacy): Navigation: Start > Settings > Ease of Access > Text cursor.; The color menu is labeled “Suggested text cursor indicator colors.”; The custom-color button is labeled “Pick a custom color.”; The indicator size slider is labeled “Change text cursor indicator size.”; The indicator switch is labeled “Turn on text cursor indicator.”; The thickness slider is labeled “Change text cursor thickness.”


### WB.09.05 — Accessibility > Magnifier

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Windows 11: Start > Settings > Accessibility > Magnifier`; `Windows 11: Start > Settings > Accessibility > Magnifier > View`; `Settings > Accessibility > Magnifier`

ms-settings URIs: `ms-settings:easeofaccess-magnifier`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/accessibility/windows/magnifier/use-magnifier-to-make-things-on-the-screen-easier-to-see); [Official support article (T1)](https://support.microsoft.com/en-gb/accessibility/windows/magnifier/use-magnifier-to-make-things-on-the-screen-easier-to-see); [Official support article (T1)](https://support.microsoft.com/en-us/accessibility/windows/magnifier/keyboard-shortcuts-for-magnifier)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.09.05.o01 | Open Magnifier settings | Link |  | — |  | — |
| WB.09.05.o02 | Magnifier | Toggle switch | On or off | Not stated | Turns Magnifier on or off. | Found at Settings > Accessibility. |
| WB.09.05.o03 | Zoom level | Buttons; Narrator announces “Zoom out, button” and “Zoom in, button” | Zoom out or Zoom in | Not stated | Chooses how much Magnifier enlarges the screen when it turns on. | Adjustable in Magnifier settings; Narrator and keyboard users can use the buttons. |
| WB.09.05.o04 | Zoom increments | Dropdown menu | Values not listed | Not stated | Sets how much the screen enlargement level changes when zooming in or out. | Adjustable in Magnifier settings. |
| WB.09.05.o05 | Start Magnifier after sign in | Toggle switch | On or off | Not stated | Controls whether Magnifier starts automatically after sign in. | Under Magnifier settings; access via “Show more settings” is described for Narrator and keyboard users. |
| WB.09.05.o06 | Start Magnifier before sign in | Toggle switch | On or off | Not stated | Controls whether Magnifier starts automatically before sign in. | Under Magnifier settings; access via “Show more settings” is described for Narrator and keyboard users. |
| WB.09.05.o07 | Smooth edges of images and text | Toggle switch | On or off | Not stated | Turns edge smoothing on or off. | Found in Magnifier settings. |
| WB.09.05.o08 | View | View selector; Narrator instructions refer to the current view setting | Full screen view, docked view, lens view | Not stated | Changes the Magnifier view. | Under Magnifier settings. The page gives keyboard shortcuts for choosing and cycling views. |
| WB.09.05.o09 | Have my Magnifier follow | Checkboxes | Mouse pointer; Keyboard focus; Text cursor; Narrator cursor. Any combination can be selected. | Not stated | Selects which of these items Magnifier follows. | Under Magnifier settings > View. |
| WB.09.05.o10 | Keep the mouse pointer | Dropdown menu | Options not listed | Not stated | Selects how Magnifier behaves when the mouse pointer moves around the screen. | Under Magnifier settings > View. |
| WB.09.05.o11 | Keep the text cursor | Dropdown menu | Options not listed | Not stated | Selects how Magnifier behaves when the text cursor moves around the screen. | Under Magnifier settings > View. |
| WB.09.05.o12 | Show more settings | Expandable control; Narrator announces “Show more settings collapsed” | Expanded or collapsed | Collapsed in the narrated instruction | Expands settings so the sign-in options can be reached. | Mentioned in the Narrator and keyboard instructions for sign-in settings. |
| WB.09.05.o13 | More | Button or control | Not stated | Not stated | Narrator users press Enter on this control to reach the “Have my Magnifier follow” checkboxes or the pointer and text cursor settings. | Described in the Narrator and keyboard instructions under View. |
| WB.09.05.o14 | Zoom each display independently | Not stated | On or off; when off, Magnifier uses span mode. | On (the default). | When on, zoom shortcuts affect only the display the pointer is on. When off, one zoom level spans all displays and viewport locking is unavailable. | Described in the context of working with multiple monitors. |
| WB.09.05.o15 | Lock view shortcut | Not stated | Ctrl+Alt+P is given as an alternate lock shortcut. | Not stated. | The alternate shortcut can be chosen under this setting. | Applies to the alternate viewport-lock shortcut. |
| WB.09.05.o16 | Reading shortcut | Not stated | The modifier key can be changed; no choices are listed. | Ctrl+Alt is the default modifier key. | Changes the modifier key used for reading commands. | Located at Settings > Accessibility > Magnifier. |

- Keyboard shortcuts: For keyboard navigation in Magnifier settings, the page describes using Tab and Shift + Tab to move between controls, Spacebar to toggle options or activate buttons, and Alt + Down arrow followed by the Up and Down arrow keys and Enter to choose dropdown values.; The page says Magnifier settings can be opened with the Windows key, but does not identify a button or other control on the Settings page.; Windows logo key + Ctrl + M opens the Magnifier settings view.
- SKU / hardware / version conditions: Zoom each display independently and viewport locking are described for use with multiple monitors.
- Windows 10 differences (legacy): The Windows 10 on/off control is labeled “Turn on Magnifier.”; The Windows 10 shortcut list includes lens resizing with Ctrl+Alt+R (mouse) and Shift+Alt+arrow keys (keyboard); these are not listed in the Windows 11 shortcut section.; The Windows 10 zoom control is labeled “Change zoom level”; the Windows 11 article calls it “Zoom level.”; The Windows 10 zoom dropdown is labeled “Change zoom increments”; the Windows 11 article calls it “Zoom increments.”; Windows 10 calls the sign-in option “Start Magnifier before sign in for everyone”; Windows 11 calls it “Start Magnifier before sign in.”; Windows 10 describes some controls as checkboxes or as selecting or unselecting them; the Windows 11 article describes them as switches and uses toggle terminology.; Windows 10 settings path: Start > Settings > Ease of Access > Magnifier.


### WB.09.06 — Accessibility > Color filters

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Accessibility`

ms-settings URIs: `ms-settings:easeofaccess-colorfilter`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/use-color-filters-in-windows-43893e44-b8b3-2e27-1a29-b0c15ef0e5ce)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.09.06.o01 | Color filters | Settings page |  | — | Opens the color filter settings. | — |
| WB.09.06.o02 | Color filter | List | A color filter from the list; individual choices are not named on the page. | — | Select a filter. The page suggests trying each filter to find one that suits you. | Turn on the Color filters switch first. |
| WB.09.06.o03 | Go to Color filters | Link |  | — | Goes to Color filters settings. | — |

- Keyboard shortcuts: Windows logo key+U
- Windows 10 differences (legacy): For previous versions of Windows 10, the navigation is Start > Settings > Ease of Access > Color & high contrast.; Windows 10 navigation: Start > Settings > Ease of Access > Color filters.; Windows 10 toggle label: “Turn on color filters”.


### WB.09.07 — Accessibility > Contrast themes

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Ease of Access > High contrast`

ms-settings URIs: `ms-settings:easeofaccess-highcontrast`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/turn-high-contrast-mode-on-or-off-in-windows-909e9d89-a0f9-a3a9-b993-7a6dcee85025)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.09.07.o01 | Turn on high contrast | Toggle |  | — | Turn on or off high contrast mode. | The page says Windows 11 handles high contrast mode through contrast themes. |
| WB.09.07.o02 | Theme | Drop-down menu | Choose a theme; the page does not list the choices. | — | Choose a theme. Colors for things like links and button text can be customized. | After turning on the toggle under “Turn on high contrast”. |
| WB.09.07.o03 | Apply | Button |  | — | Apply the selected theme and customized colors. | After choosing a theme and customizing colors. |
| WB.09.07.o04 | Change color contrast in Windows | Link |  | — | Links to information about contrast themes. | — |

- Restart / sign-out: From the sign-in screen, select the “Ease of access” button in the lower-right corner, then turn on the toggle under “High Contrast”.
- Keyboard shortcuts: Left Alt + left Shift + Print Screen toggles high contrast on or off.
- Windows 10 differences (legacy): The page lists Windows 10 as applicable but does not provide distinct Windows 10-only steps.


### WB.09.08 — Accessibility > Narrator

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Settings > Accessibility > Narrator`; `Settings > Accessibility > Narrator > Use a braille display with Narrator`

ms-settings URIs: `ms-settings:easeofaccess-narrator`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/accessibility/windows/narrator/complete-guide-to-narrator)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.09.08.o01 | Add a natural voice | Button | Preferred HD voice; the page mentions Andrew and Ava for English (United States), and HD voices in French and Chinese. | — | Downloads the selected voice. Voices can be switched and their speed adjusted in Narrator settings. | The page says HD voices are available in English (United States), with French and Chinese HD voices also mentioned. |
| WB.09.08.o02 | Use a braille display with Narrator | Settings option / destination in Settings |  | — | The page says to navigate here to download and install a braille package. Braille output is needed for the braille viewer to work. | Required for braille output and the braille viewer. |
| WB.09.08.o03 | option to obtain Image Descriptions | Option |  | — | The page says this option must be activated to use image descriptions. | The page does not give a Settings path or a more specific UI label for this option. |
| WB.09.08.o04 | Narrator key | Modifier-key setting | Caps lock and Insert | Both Caps lock and Insert serve as the Narrator key by default. | The page says you can change the Narrator key in Narrator settings. | — |

- Keyboard shortcuts: Windows logo key+Ctrl+N — open Narrator settings.
- SKU / hardware / version conditions: HD voices are described as available in English (United States); the page also mentions HD voices in French and Chinese.; The page says natural voices support on-device text-to-speech after download and do not require an internet connection.
- Windows 10 differences (legacy): The Windows 10 section gives the settings path as Settings > Ease of Access > Narrator.; The Windows 10 section names the Narrator settings section “Change what you hear when reading and interacting” for capitalization behavior.; The Windows 10 section says the keyboard defaults to the Standard layout and that you can choose the Legacy layout in Narrator settings.; The Windows 10 section says the toggle under “Help make Narrator better” enables additional diagnostic and performance data; it says to restart Narrator after turning it on.


### WB.09.09 — Accessibility > Audio

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Accessibility > Audio`; `スタート > 設定 > アクセシビリティ > オーディオ`; `Démarrer > Paramètres > Accessibilité > Audio`

ms-settings URIs: `ms-settings:easeofaccess-audio`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/accessibility/windows/make-windows-easier-to-hear); [Official support article (T1)](https://support.microsoft.com/ja-jp/windows/windows-%E3%81%AE%E9%9F%B3%E5%A3%B0%E3%82%92%E8%81%9E%E3%81%8D%E5%8F%96%E3%82%8A%E3%82%84%E3%81%99%E3%81%8F%E3%81%99%E3%82%8B-9c18cfdc-63be-2d47-0f4f-5b00facfd2e1); [Official support article (T1)](https://support.microsoft.com/fr-fr/windows/simplifier-l-%C3%A9coute-de-windows-9c18cfdc-63be-2d47-0f4f-5b00facfd2e1)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.09.09.o01 | Mono audio | Toggle |  | — | Converts stereo sound into a single channel so you can hear everything, even when using just one headphone. | — |
| WB.09.09.o02 | Flash my screen during audio notifications | Setting with choices | The title bar of the active window; the active window; the entire screen | — | Displays audio alerts visually when a notification arrives, instead of relying on sound alone. | — |
| WB.09.09.o03 | Mono オーディオ | トグル | オン／オフ | — | ステレオ サウンドを 1 つのチャンネルに変換し、ヘッドフォンを 1 つだけ使用している場合でもすべての音を聞けるようにする。 | — |
| WB.09.09.o04 | オーディオ通知中に画面をフラッシュする | 選択メニュー | 作業中のウィンドウのタイトル バー、作業中のウィンドウ、画面全体 | — | オーディオ通知を受け取った際に、選択した場所を点滅させて視覚的に表示する。 | — |
| WB.09.09.o05 | Audio Mono | Bouton bascule |  | — | Convertit le son stéréo en mono sur un seul canal afin que vous puissiez tout entendre même si vous n’utilisez qu’un seul écouteur. | — |
| WB.09.09.o06 | Flasher mon écran pendant les notifications audio | Choix de mode d’affichage | Clignotement de la fenêtre active ; de la barre de titre active ; ou de la totalité de l’écran. | — | Signale visuellement l’arrivée d’une notification audio, plutôt que par une simple tonalité. | — |

- Windows 10 differences (legacy): Chemin Windows 10 : Démarrer > Paramètres > Options d’ergonomie > Audio.; La section des alertes est intitulée « Afficher les alertes audio visuellement ».; Le bouton bascule est indiqué sous le libellé « Activer l’audio mono ».; The mono audio toggle is described as the toggle under "Turn on mono audio".; The path is Start > Settings > Ease of Access > Audio.; The visual audio-alert setting is labeled "Show audio alerts visually".; オーディオ警告の視覚表示は「オーディオの警告を視覚的に表示する」と記載されている。; モノラル オーディオのトグルは「モノラル オーディオをオンにする」の下にあると記載されている。


### WB.09.10 — Accessibility > Captions

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Settings > Accessibility > Captions`; `Live captions Settings > Preferences > Caption style > Accessibility settings for Captions`; `Live captions > Settings > Preferences > Caption style opens the Accessibility settings for Captions`

ms-settings URIs: `ms-settings:easeofaccess-closedcaptioning`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/use-live-captions-to-better-understand-audio-b52da59c-14b8-4031-aeeb-f6a47e6055df); [Official support article (T1)](https://support.microsoft.com/en-us/accessibility/windows/use-live-captions-to-better-understand-audio); [Official support article (T1)](https://support.microsoft.com/en-gb/accessibility/windows/use-live-captions-to-better-understand-audio?nochrome=true)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.09.10.o01 | Live captions | toggle |  | — | Turn on to enable live captions. | The first time it is turned on, live captions asks for consent to process voice data on the device and prompts you to download language files for on-device speech recognition. |
| WB.09.10.o02 | Caption style | dropdown | Built-in styles, including Default | — | Choose a built-in style, or select Edit to create a custom style. The Default built-in style uses colors appropriate to the device’s dark or light mode. | — |
| WB.09.10.o03 | Edit | button |  | — | Opens the option to create a custom caption style. | Available under Caption style. |

- Confirmations / errors: The page refers users to “Troubleshooting live captions or voice access setup issues” for a first-run download error.; When Live captions is turned on for the first time, a consent prompt and a prompt to download language files appear.; When live captions is turned on for the first time, it asks for consent to process voice data on the device and prompts the user to download language files.; When live captions is turned on for the first time, it asks for consent to process voice data on the device and prompts the user to download language files. If the language is unavailable, another language can be used during setup.
- Keyboard shortcuts: Windows + Ctrl + L turns on live captions.; Windows Ctrl+L
- SKU / hardware / version conditions: Live captions is available in Windows 11, version 22H2 and later.; Translation is available on Copilot+ PCs running Windows 11, version 24H2 and later.
- Windows 10 differences (legacy): No Windows 10 differences are stated.


### WB.09.11 — Accessibility > Speech / Voice access

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Settings > Accessibility > Speech`; `Accessibility > Speech (the page section opened by the “Automatic startup setting” option)`; `Start menu > Settings > Accessibility > Speech`

ms-settings URIs: `ms-settings:easeofaccess-speechrecognition`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/accessibility/windows/voice-access/voice-access-frequently-asked-questions-faqs); [Official support article (T1)](https://support.microsoft.com/en-us/accessibility/windows/voice-access/get-started-with-voice-access); [Official support article (T1)](https://support.microsoft.com/en-us/accessibility/windows/voice-access/use-voice-access-to-set-up-and-log-in-to-your-pc)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.09.11.o01 | Voice access | Feature/entry point |  | — | Voice access can be started from Settings > Accessibility > Speech. | Available in Windows 11, version 22H2 and later. |
| WB.09.11.o02 | Start voice access before signing in to your PC | Option (control type not specified) |  | — | Selecting this option sets voice access to start automatically every time. | For using voice access to log in, available starting with Windows 11, version 23H2. |
| WB.09.11.o03 | Automatic startup setting | Voice access Settings menu option; the page does not state the exact control label on the Windows Settings page. | Enable voice access to launch automatically before or after logging into the computer. | — | Opens the Speech section under Accessibility, where automatic startup can be enabled. | — |
| WB.09.11.o04 | Turn off voice access | Voice access Settings menu option; not identified as a control on the Windows Settings page. |  | — | Removes the voice access UI from the top of the screen and closes voice access. To launch voice access again, go to Settings > Accessibility > Speech. | — |
| WB.09.11.o05 | Start voice access before sign-in | Checkbox |  | — | When checked, voice access starts automatically before sign-in. | The page says to select Settings > Accessibility > Speech from the Start menu and check this option to have voice access on every time. |

- SKU / hardware / version conditions: Using voice access to log in is available starting with Windows 11, version 23H2.; Voice access is available in Windows 11, version 22H2 and later.


### WB.09.12 — Accessibility > Keyboard

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Accessibility > Keyboard`

ms-settings URIs: `ms-settings:easeofaccess-keyboard`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/accessibility/windows/make-your-mouse-keyboard-and-other-input-devices-easier-to-use)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.09.12.o01 | On-screen keyboard | Switch | On or off | — | Turn it on to select keys using the mouse or another pointing device, such as a joystick, or to use a single switch to cycle through the keys on the screen. | — |
| WB.09.12.o02 | Sticky keys | Switch | On or off | — | Turn it on if you have trouble pressing two keys at the same time. It lets you press commands that use multiple keys, such as Ctrl+S, one key at a time. | — |
| WB.09.12.o03 | Toggle keys | Switch | On or off | — | Turn it on to play a sound whenever you press the Caps lock, Num lock, or Scroll lock keys. | — |
| WB.09.12.o04 | Filter keys | Switch | On or off | — | Turn it on to set keyboard sensitivity so it can ignore brief or repeated keystrokes. | — |
| WB.09.12.o05 | Underline access keys | Switch | On or off | — | Turn it on to underline access keys when they're available. | — |
| WB.09.12.o06 | Use the Print screen key to open screen capture | Switch | On or off | — | Turn it on to use the Prt Scr button to open screen snipping. | — |
| WB.09.12.o07 | Use voice recognition in Windows | Link |  | — | The page says you can also control your PC with your voice using Windows Speech Recognition and links to learn more. | — |

- Keyboard shortcuts: Windows logo + Ctrl + O; Windows logo + U
- Windows 10 differences (legacy): The Windows 10 label is "Use the On-Screen Keyboard".; The keyboard settings path is Start > Settings > Ease of Access > Keyboard.; Windows 10 uses "Change how keyboard shortcuts work" to underline access keys when available.; Windows 10 uses "Print Screen shortcut" to open screen snipping.; Windows 10 uses the labels "Sticky Keys", "Toggle Keys", and "Filter Keys".


### WB.09.13 — Accessibility > Mouse (Mouse keys)

Status: PARTIAL. Official source confirms the page exists but documents few or none of its controls.

Navigation paths (as stated in sources): `Start > Settings > Accessibility > Mouse`; `Settings > Accessibility > Mouse > Mouse keys speed`; `Windows 11: Start > Settings > Accessibility > Mouse (under Interaction)`; `Windows 11: Settings > Accessibility > Mouse > Mouse keys speed`; `Start > Settings > Accessibility > Interaction > Mouse`

ms-settings URIs: `ms-settings:easeofaccess-mouse`

Sources: [Official support article (T1)](https://support.microsoft.com/help/14204); [Official support article (T1)](https://support.microsoft.com/en-us/accessibility/windows/use-mouse-keys-to-move-the-mouse-pointer); [Official support article (T1)](https://support.microsoft.com/en-us/windows/use-mouse-keys-to-move-the-mouse-pointer-9e0c72c8-b882-7918-8e7b-391fd62adf33)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.09.13.o01 | Mouse keys | Toggle button | On or off | — | When turned on, the numeric keypad can be used instead of the mouse to move the pointer and perform mouse actions. | Found under the Interaction section of Mouse settings. |
| WB.09.13.o02 | Mouse keys speed | Sub-page |  | — | Used to adjust options such as pointer speed and whether the computer makes a sound when Mouse keys is turned on. | — |

- Keyboard shortcuts: Click with the left button selected: 5. Right-click with the right button selected: 5. Double-click with the left button selected: +.; Drag: point to the item and press 0. Drop: point to the destination and press .; Move pointer: 7 (up-left), 8 (up), 9 (up-right), 4 (left), 6 (right), 1 (down-left), 2 (down), 3 (down-right).; Select active mouse button: / (left), ✲ (both), - (right).
- Windows 10 differences (legacy): For the sound and pointer-speed options, the page directs users to the Ease of Access Center, under “Control the mouse with the keyboard,” and then “Set up Mouse keys.”; Navigation: Start > Settings > Ease of access > Mouse.; Settings path: Start > Settings > Ease of access > Mouse.; The toggle is labeled “Control your mouse with a keypad.”; To change options such as pointer speed or sound, the page directs users to Ease of Access Center > Control the mouse with the keyboard > Set up Mouse keys.; Toggle label: “Control your mouse with a keypad.”; Windows 10 instructions also say to open Ease of Access Center, under “Control the mouse with the keyboard”, and click “Set up Mouse Keys” to change options.; Windows 10 path: Start > Settings > Ease of access > Mouse.; Windows 10 toggle label: “Control your mouse with a keypad”.


### WB.09.14 — Accessibility > Eye control

Status: PARTIAL. Official source confirms the page exists but documents few or none of its controls.

Navigation paths (as stated in sources): `Start > Settings > Accessibility > Interaction > Eye control`; `Settings > Accessibility > Eye control`

ms-settings URIs: `ms-settings:easeofaccess-eyecontrol`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/accessibility/windows/eye-control/eye-control-basics-in-windows); [Official support article (T1)](https://support.microsoft.com/help/4043921)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.09.14.o01 | Eye control | Switch | On or off | — | Turning it on makes the launchpad appear on the screen. | — |
| WB.09.14.o02 | Open Eye control settings | Link |  | — | Opens Eye control settings. | — |

- Windows 10 differences (legacy): For Windows 10, the path is Settings > Ease of Access > Eye control.; The Windows 10 section gives the path as Settings > Ease of Access > Interaction > Eye control and calls the switch option “Use eye control.”; Windows 10 requires the Fall Creator’s Update or later (version 1709 or higher); earlier versions do not have eye control.


## WB.10 — Privacy & security


### WB.10.01 — Privacy & security > Windows Security

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `To open the Windows Security app: Windows Security`; `Open Windows Security`; `In the Windows Security app: Virus & threat protection > Quick scan`; `In the Windows Security app: Virus & threat protection > Manage settings`; `In File Explorer, right-click a file or folder, then select Scan with Microsoft Defender`

ms-settings URIs: `ms-settings:windowsdefender`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/security/windows-security/windows-security-app-overview); [Official support article (T1)](https://support.microsoft.com/en-us/windows/security/windows-security/stay-protected-with-the-windows-security-app); [Official support article (T1)](https://support.microsoft.com/Windows/Security/windows-security/stay-protected-with-the-windows-security-app)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.10.01.o01 | Virus & threat protection | Protection area |  | — | Monitor threats, run scans, get updates to help detect the latest threats, and access real-time protection settings. | — |
| WB.10.01.o02 | Quick scan | Scan option |  | — | Run a quick scan. | Available under Virus & threat protection. |
| WB.10.01.o03 | Full scan | Scan option |  | — | Run a full scan. | Available under Virus & threat protection. |
| WB.10.01.o04 | Custom scan | Scan option |  | — | Run a custom scan. | Available under Virus & threat protection. |
| WB.10.01.o05 | Real-time protection settings | Settings |  | — | Access real-time protection settings. | Available under Virus & threat protection. |
| WB.10.01.o06 | Account protection | Protection area |  | — | Access sign-in options and account settings, including Windows Hello and dynamic lock. | — |
| WB.10.01.o07 | Windows Hello | Sign-in option |  | — |  | Mentioned under Account protection. |
| WB.10.01.o08 | dynamic lock | Account setting |  | — |  | Mentioned under Account protection. |
| WB.10.01.o09 | Firewall & network protection | Protection area |  | — | View and manage Windows Firewall status, turn the firewall on or off, and access advanced settings for different network types. | — |
| WB.10.01.o10 | Windows Firewall | Firewall | On or off | — | View and manage its status; turn it on or off. | Managed through Firewall & network protection. |
| WB.10.01.o11 | Advanced settings | Settings |  | — | Access advanced settings for different network types. | Available through Firewall & network protection. |
| WB.10.01.o12 | App & browser control | Protection area |  | — | Offers Microsoft Defender SmartScreen settings to help protect against potentially dangerous apps, files, sites, and downloads. Also provides exploit protection, protection from potentially unwanted apps, and phishing protection settings. | — |
| WB.10.01.o13 | Microsoft Defender SmartScreen | Protection setting |  | — | Helps protect against potentially dangerous apps, files, sites, and downloads. | Available under App & browser control. |
| WB.10.01.o14 | Exploit protection | Protection setting |  | — |  | Available under App & browser control. |
| WB.10.01.o15 | Protection from potentially unwanted apps | Protection setting |  | — |  | Available under App & browser control. |
| WB.10.01.o16 | Phishing protection | Protection setting |  | — |  | Available under App & browser control. |
| WB.10.01.o17 | Device security | Protection area |  | — | Manage security features built into the Windows device, like core isolation, TPM, and secure boot. | — |
| WB.10.01.o18 | Core isolation | Device security feature |  | — |  | Mentioned under Device security. |
| WB.10.01.o19 | TPM | Device security feature |  | — |  | Mentioned under Device security. |
| WB.10.01.o20 | Secure boot | Device security feature |  | — |  | Mentioned under Device security. |
| WB.10.01.o21 | Device performance & health | Protection area |  | — | View the device’s performance health report. | — |
| WB.10.01.o22 | Family options | Protection area |  | — | Access tools to manage your family’s digital life. | — |
| WB.10.01.o23 | Protection history | Protection area |  | — | Review actions taken by Defender Antivirus, potentially unwanted apps that have been removed, and key services that are turned off. | — |
| WB.10.01.o24 | Summary | Page section |  | — | Describes Windows Security as including Microsoft Defender Antivirus, Windows Firewall, and Smart App Control. | — |
| WB.10.01.o25 | Common tasks | Page section |  | — | Contains the listed malware-scanning and real-time-protection tasks. | — |
| WB.10.01.o26 | Run a malware scan manually | Page subsection |  | — | Right-click a file or folder in File Explorer and select Scan with Microsoft Defender. | On Windows 11, Show more options may be needed after right-clicking to see the scan command. |
| WB.10.01.o27 | Scan with Microsoft Defender | File Explorer context-menu command |  | — | Scans the selected file or folder. | On Windows 11, it may appear after selecting Show more options. |
| WB.10.01.o28 | Show more options | File Explorer context-menu command |  | — | May reveal Scan with Microsoft Defender after right-clicking a file or folder. | Windows 11 |
| WB.10.01.o29 | Run a quick scan | Page subsection |  | — | Recommends running a quick scan if malware or a virus is suspected. | — |
| WB.10.01.o30 | Start a quick scan | Shortcut |  | — | Offered as a shortcut to start a quick scan. | — |
| WB.10.01.o31 | Turn Defender Antivirus real-time protection on or off | Page subsection |  | — | Explains that real-time protection can be briefly stopped and will soon turn on automatically again. | — |
| WB.10.01.o32 | Manage settings | Windows Security navigation item |  | — | Opens the settings location for real-time protection under Virus & threat protection. | — |
| WB.10.01.o33 | Virus & threat protection settings | Shortcut |  | — | Offered as a shortcut to the virus and threat protection settings. | — |
| WB.10.01.o34 | Real-time protection | Toggle | On; Off | — | When off, files you open or download won't be scanned for threats. Real-time protection will soon turn on automatically again. | Located under Virus & threat protection > Manage settings. |

- Confirmations / errors: Certain pages or features might not be available if the device is managed by an organization. Contact the IT administrator for more information.; Protection-area status icons: green means the device is sufficiently protected and there aren’t any recommended actions; yellow means there is a safety recommendation; red indicates something needs immediate attention.
- Keyboard shortcuts: Start a quick scan; Virus & threat protection settings
- SKU / hardware / version conditions: Certain pages or features might not be available if the device is managed by an organization.; If another antivirus app is installed and turned on, Microsoft Defender Antivirus turns off automatically. If the other app is uninstalled, Microsoft Defender Antivirus turns back on automatically.; The page says Microsoft 365 Family or Personal subscribers get Microsoft Defender as part of their subscription.
- Windows 10 differences (legacy): On Windows 11, Show more options may be needed after right-clicking a file or folder to see Scan with Microsoft Defender.; The page applies to Windows 11 and Windows 10.; The page says that on Windows 11, Show more options may be needed after right-clicking a file or folder to see Scan with Microsoft Defender.


### WB.10.02 — Privacy & security > Find my device

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Go to account.microsoft.com/devices and sign in.`; `Select the “Find My Device” tab.`; `“Open Find my device settings” is provided as a link; the page does not give its destination or a Settings navigation path.`; `Windows 11: Start > Settings > Privacy & security > Find my device`; `Windows 10: Start > Settings > Update & Security > Find my device`

ms-settings URIs: `ms-settings:findmydevice`

Sources: [Official support article (T1)](https://support.microsoft.com/help/11579); [Official support article (T1)](https://support.microsoft.com/en-gb/account-billing/find-and-lock-a-lost-windows-device-890bf25e-b8ba-d3fe-8253-e98a12f26316) (dated 2024-03-15); [Official support article (T1)](https://support.microsoft.com/en-us/windows/find-and-lock-a-lost-windows-device-890bf25e-b8ba-d3fe-8253-e98a12f26316)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.10.02.o01 | Summary | Page section |  | Not stated. | Describes Find My Device and its setup requirements. | — |
| WB.10.02.o02 | Open Find my device settings | Link |  | Not stated. | Opens Find my device settings; the destination is not specified on this page. | — |
| WB.10.02.o03 | Find a lost Windows device | Page section |  | Not stated. | Instructions for finding a device using the Microsoft account devices page. | — |
| WB.10.02.o04 | Find My Device | Tab |  | Not stated. | Select this tab on account.microsoft.com/devices. | — |
| WB.10.02.o05 | Find | Command |  | Not stated. | Shows a map with the selected device’s location. | Choose the device you want to find. |
| WB.10.02.o06 | Lock a Windows device remotely | Page section |  | Not stated. | Instructions for remotely locking a device. | — |
| WB.10.02.o07 | Lock | Command |  | Not stated. | Starts the remote-lock flow. | Select it when the device is shown on the map. |
| WB.10.02.o08 | Next | Command |  | Not stated. | Continues after selecting Lock. | — |
| WB.10.02.o09 | Info and support | Section label |  | Not stated. | The device serial number can be found here on account.microsoft.com/devices. | — |
| WB.10.02.o10 | See also | Page section |  | Not stated. | Lists related help links. | — |
| WB.10.02.o11 | Need more help? | Page section |  | Not stated. | Provides sign-in and support guidance. | — |
| WB.10.02.o12 | Get Help | Command |  | Not stated. | Used after entering a problem in Contact Microsoft Support. | — |
| WB.10.02.o13 | Contact Support | Command |  | Not stated. | Routes the user to the best support option if more help is needed. | Select after using Get Help if more help is still needed. |
| WB.10.02.o14 | Location | Device setting | Turned on or not turned on; no UI choices are specified. | Not stated. | The feature works when location is turned on for the device, even if other users have turned off location settings for their apps. | — |
| WB.10.02.o15 | Applies To | section heading |  | — | States that Find My Device applies to Windows 10 and Windows 11 devices, including a PC, laptop, Surface, or Surface Pen. | — |
| WB.10.02.o16 | Turn on Find my device | section heading |  | — | Explains enabling the setting. | — |
| WB.10.02.o17 | You can't use | Incomplete text in the supplied page |  | — | The supplied text ends after this phrase; no restriction is stated. | — |
| WB.10.02.o18 | Find your Windows device | section heading |  | — | Describes locating a device through the Microsoft account devices page. | — |
| WB.10.02.o19 | Lock your Windows device remotely | section heading |  | — | Describes remotely locking the device. | — |
| WB.10.02.o20 | Find my device settings | Link | Not stated | — | Opens the Find my device settings; the destination path is not stated. | Find My Device must be turned on before it can be used. |
| WB.10.02.o21 | Administrator | Account type requirement | Administrator | — | Required account type for Find My Device. | The page says the account type needs to be Administrator. |

- Confirmations / errors: Any time you attempt to locate the device, users using it will see a notification in the notification area.; If you cannot sign in to your Microsoft account, the page recommends using the sign-in helper tool.; Microsoft support agents are not allowed to send password reset links or access and change account details.; Support agents cannot send password reset links or access and change account details.; The page recommends reporting a stolen device to local law enforcement and the insurance company.; When you attempt to locate the device, users using it will see a notification in the notification area.
- SKU / hardware / version conditions: Applies to Windows 10 and Windows 11.; Find My Device must be turned on before it can be used.; Sign in to the device with a Microsoft personal account.; The account type must be Administrator.; The feature is for Windows 10 or Windows 11 devices.; The feature works for Windows devices such as a Surface, another PC or laptop, or a Surface Pen.; The page says it works for any Windows device, including a Surface or other PC or laptop, or a Surface Pen.; Use a Microsoft personal account; the account type must be Administrator.
- Windows 10 differences (legacy): The Windows 10 Settings path is Start > Settings > Update & Security > Find my device; the Windows 11 path uses Privacy & security.; The page applies to both Windows 10 and Windows 11 and states no differences between them.


### WB.10.03 — Privacy & security > Device encryption

Status: PARTIAL. Official source confirms the page exists but documents few or none of its controls.

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/security/encryption/bitlocker-overview)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.10.03.o01 | Device Encryption | Feature/functionality; no Settings control or page UI is described | Designed for simplicity of use; usually enabled automatically | Usually enabled automatically | Protects data by encrypting drives. The page says its recovery key is automatically saved to a Microsoft account or work or school account before protection is activated when Device Encryption is automatically enabled. If an administrator manually activates it, the key is automatically saved to their Microsoft account or work or school account before protection is activated. | The recovery-key saving behavior depends on whether Device Encryption is automatically enabled or manually activated by an administrator. |
| WB.10.03.o02 | BitLocker Drive Encryption | Feature/functionality; no Settings control or page UI is described | Designed for advanced scenarios; allows manual encryption of drives | — | When an administrator manually activates it, the user activating BitLocker selects where to save the recovery key. | The recovery-key behavior described applies when an administrator manually activates BitLocker Drive Encryption. |

- Confirmations / errors: BitLocker might require the recovery key if it detects possible unauthorized access or changes to hardware, firmware, or software. The page does not give an exact prompt or error-message label.; Without the recovery key, the user will not be able to access the drive if BitLocker requires it.
- SKU / hardware / version conditions: The page says hardware, firmware, or software changes can cause BitLocker to require the recovery key. It gives no device-eligibility or hardware-version requirements for a Device Encryption page.


### WB.10.05.01 — Privacy & security > General

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Privacy & security > General`

ms-settings URIs: `ms-settings:privacy`, `ms-settings:privacy-general`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/privacy/general-privacy-settings-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.10.05.01.o01 | Let apps show me personalized ads by using my advertising ID | Setting (control type not specified) |  | — | Apps can use the advertising ID to provide more relevant advertising and personalized experiences. Turning it off does not reduce the number of ads, but ads may be less interesting and relevant. Turning it back on resets the advertising ID. | Applies to Windows apps using the Windows advertising identifier; it does not apply to other methods of interest-based advertising. |
| WB.10.05.01.o02 | Let websites show me locally relevant content by accessing my language list | Setting (control type not specified) |  | — | Windows can share information about your preferred language list with websites so they can offer content in relevant languages. | — |
| WB.10.05.01.o03 | Let Windows improve Start and search results by tracking app launches | Setting (control type not specified) |  | — | Windows can use apps you launch to personalize the Start menu and provide access to your most-used apps in Start and search. | — |
| WB.10.05.01.o04 | Show me suggested content in the Settings app | Setting (control type not specified) |  | — | Microsoft can show suggestions in Settings to help discover features, content, and apps. The page says to turn this setting off to stop suggested content. | — |

- Confirmations / errors: The General privacy settings page is being replaced by the Recommendations & offers privacy settings page in more recent versions of Windows 11. If the latest Windows Update is not installed, the General page might still appear.; Turning the advertising ID off does not reduce the number of ads; turning it back on resets the advertising ID.
- Windows 10 differences (legacy): Advertising ID path: Start > Settings > Privacy > General. The label is “Let apps use advertising ID to make ads more interesting to you based on your app activity”; earlier Windows 10 versions called it “relevant ads.”; App launch tracking path: Start > Settings > Privacy > General. Its label is “Let Windows track app launches to improve Start and search results.”; Suggested content path: Start > Settings > Privacy > General. The label is “Show me suggested content in the Settings app.”; Website language setting path: Start > Settings > Privacy > General. Its label is “Let websites provide locally relevant content by accessing my language list.”


### WB.10.05.02 — Privacy & security > Speech / Inking & typing personalization

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Privacy & security > Speech`; `Start > Settings > Privacy & security > Inking & typing personalization`; `開始 > 設定 > 隱私 & 安全 > 言論`; `開始 > 設定 > 隱私 & 安全 > 描線 & 打字個人化`

ms-settings URIs: `ms-settings:privacy-speech`, `ms-settings:privacy-speechtyping`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/privacy/speech-voice-activation-inking-typing-and-privacy); [Official support article (T1)](https://support.microsoft.com/ko-KR/Windows/privacy/speech-voice-activation-inking-typing-and-privacy); [Official support article (T1)](https://support.microsoft.com/zh-hk/windows/%E8%AA%9E%E9%9F%B3-%E8%AA%9E%E9%9F%B3%E5%95%9F%E5%8B%95-%E7%AD%86%E8%B7%A1-%E8%BC%B8%E5%85%A5%E8%88%87%E9%9A%B1%E7%A7%81%E6%AC%8A-149e0e60-7c93-dedd-a0d8-5731b71a4fef)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.10.05.02.o01 | Online speech recognition | Toggle | On or Off | — | When turned off, only device-based speech recognition features are available. Online speech recognition uses Microsoft cloud-based services; voice data is sent to Microsoft to provide the service and create text transcriptions. | — |
| WB.10.05.02.o02 | Custom inking and typing word list | Toggle | On or Off | — | Turning this off clears your custom word list. | The custom word list is created from words you type or write, is associated with your Microsoft account, and can be used across Microsoft products. |
| WB.10.05.02.o03 | How does Microsoft protect my privacy while improving its speech recognition technology? | Link |  | — | The page refers to this for more information about speech recognition privacy. | — |
| WB.10.05.02.o04 | 線上語音辨識 | 切換開關 | 開啟／關閉 | — | 開啟時使用 Microsoft 雲端服務進行語音辨識。關閉時僅能使用裝置上的語音辨識功能，例如旁白。 | — |
| WB.10.05.02.o05 | 自訂描線與輸入詞彙表 | 切換開關 | 開啟／關閉 | — | Windows 會根據打字或書寫的字詞建立自訂單字清單，以提升打字和墨水的準確度。清單與 Microsoft 帳戶相關聯，且可跨 Microsoft 產品使用。關閉此功能會清除自訂詞彙列表。 | — |

- Confirmations / errors: Turning off Custom inking and typing word list clears your custom word list.
- Windows 10 differences (legacy): Inking and typing personalization path: Start > Settings > Privacy > Inking & typing personalization. Under Getting to know you, turn the setting Off; turning it off clears the custom word list.; Some Windows 10 versions offer the buttons Start contributing my voice clips and Stop contributing my voice clips under Help make online speech recognition better. If the option is unavailable, voice clips aren’t contributed.; Speech settings path: Start > Settings > Privacy > Speech. The page says to turn Online speech recognition On or Off; it does not state a default.; The page states that support for Windows 10 ended on October 14, 2025.; Windows 10 的描線與打字個人化設定路徑為「開始 > 設定 > 隱私 > 墨線 & 打字個人化」。在「了解你」裡可將設定關閉；關閉會清除自訂詞彙列表。; Windows 10 的語音設定路徑為「開始 > 設定 > 隱私 > 語音」。部分版本另有「幫助改善線上語音辨識」選項，可選「開始貢獻我的語音片段」或「別再分享我的語音片段」。若沒有這個選項，語音片段就不會被貢獻。


### WB.10.05.03 — Privacy & security > Diagnostics & feedback

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Privacy & security > Diagnostics & feedback`

ms-settings URIs: `ms-settings:privacy-feedback`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/privacy/diagnostics-feedback-and-privacy-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.10.05.03.o01 | Diagnostic data | Setting with selectable options | Required; Optional | — | Optional diagnostic data includes Required diagnostic data. The page says the device remains secure and operates normally when sending only Required diagnostic data. | If the options are unavailable, the device may be managed by a workplace or organization. |
| WB.10.05.03.o02 | Tailored experiences | Setting | On; Off | — | When on, Windows diagnostic data is used to offer personalized tips, ads, and recommendations. With Required diagnostic data selected, personalization is based on device information, settings, capabilities, and whether the device is performing properly. With Optional selected, it also uses information about app and feature use and device health. | The page says Tailored experiences is being replaced by Personalized offers in more recent Windows versions. |
| WB.10.05.03.o03 | Improve inking and typing | Setting | On; Off | — | When on, Microsoft collects samples of typed or written content to improve handwriting recognition, autocompletion, next word prediction, and spelling correction. The page also refers to this setting as “Improve inking & typing.” | The page says the setting is not available in previous versions of Windows, where this data is collected when Diagnostic data is set to Full. |
| WB.10.05.03.o04 | View diagnostic data | Setting | On; Off | — | Turn it on to use the Diagnostic Data Viewer. | — |
| WB.10.05.03.o05 | Open Diagnostic Data Viewer | Button |  | — | Opens the Diagnostic Data Viewer. It displays data available while the viewer is running, not diagnostic data history. | The View diagnostic data setting must be On. |
| WB.10.05.03.o06 | Delete diagnostic data | Setting section |  | — | Provides an option to delete diagnostic data for the device. | — |
| WB.10.05.03.o07 | Delete | Button |  | — | Deletes diagnostic data for the device. It does not delete diagnostic data associated with the Microsoft account or stop diagnostic data from being sent to Microsoft. | An organization may have a copy of the device’s diagnostic data if it enrolled the device in services that rely on that data. |
| WB.10.05.03.o08 | Microsoft privacy dashboard | Link |  | — | Provides a place to view and delete additional diagnostic data associated with the Microsoft account. | — |
| WB.10.05.03.o09 | Feedback frequency | Setting with selectable options | Options not specified | — | Adjusts how often Windows asks the user to rate or provide written feedback. | — |

- Confirmations / errors: Deleting diagnostic data from this page does not delete diagnostic data associated with the Microsoft account and does not stop diagnostic data from being sent to Microsoft.; If Diagnostic data options are unavailable, the screen may state: “Some settings are hidden or managed by your organization”.; The Diagnostic Data Viewer shows data available while it is running and does not show diagnostic data history.
- SKU / hardware / version conditions: The page says that on Windows 11 version 23H2 and newer devices in the European Economic Area, Microsoft Edge diagnostic data has been collected separately from Windows diagnostic data since March 6, 2024, and is subject to its own settings.
- Windows 10 differences (legacy): In previous versions of Windows, the “Improve inking & typing” setting is not available, and this data is collected when Diagnostic data is set to Full.; The Settings path is Start > Settings > Privacy > Diagnostics & feedback.; The Windows 10 setting is labeled “Diagnostic data viewer”; turn it on and select “Diagnostic Data Viewer.”; The page says Windows 10 is in the process of recategorizing Basic diagnostic data as Required diagnostic data and Full diagnostic data as Optional diagnostic data.; The page says that on Windows 10 version 22H2 and newer devices in the European Economic Area, Microsoft Edge diagnostic data has been collected separately from Windows diagnostic data since March 6, 2024, and is subject to its own settings.


### WB.10.05.04 — Privacy & security > Activity history

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Privacy & security > Activity history`

ms-settings URIs: `ms-settings:privacy-activityhistory`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/privacy/windows-activity-history-and-your-privacy)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.10.05.04.o01 | Store my activity history on this device | Toggle | On; Off | — | When enabled, activity history is stored on the device. Switching it Off stops saving activity history locally. | — |
| WB.10.05.04.o02 | Clear activity history for this account | Button | Clear history | — | Clears the activity history stored on the device. The page also says previously cloud-stored activity history can be deleted with this button; otherwise, that data is automatically deleted within 30 days of its last sync. | The Clear history button appears next to this label. |
| WB.10.05.04.o03 | Open Activity history settings | Link |  | — | Opens Activity history settings. | — |
| WB.10.05.04.o04 | Option to send activity history to Microsoft | Option (deprecated in the described Windows 11 versions) |  | — | The option was deprecated in Windows 11 23H2 and 22H2 by the January 23, 2024-KB5034204 update. Previous Windows 11 versions may still show it; if enabled, activity history data will still be sent to Microsoft. The page says the Microsoft account settings described do not allow activity history to be sent to Microsoft. | Applies to previous Windows 11 versions before the specified update. |

- Windows 10 differences (legacy): In previous Windows 10 versions, the setting was called “Let Windows collect my activities from this PC.”; The option to send activity history to Microsoft was deprecated from Windows 10 22H2 by the January 23, 2024-KB5034203 update; previous versions may still show it.; Windows 10 clear-history path and button are labeled “Clear activity history” and “Clear”; Windows 11 uses “Clear activity history for this account” and “Clear history.”; Windows 10 path: Start > Settings > Privacy > Activity history.; Windows 10 uses a checkbox for “Store my activity history on this device”; Windows 11 uses a switch.


### WB.10.05.05 — Privacy & security > Search permissions / Searching Windows

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Privacy & security > Search permissions`; `Start > Settings > Privacy & security > Searching Windows`; `Settings > Privacy & security > Searching Windows > Advanced indexing options`

ms-settings URIs: `ms-settings:search-permissions`, `ms-settings:cortana-windowssearch`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/windows-search-and-privacy-99fb8251-7260-1cd6-1bbb-15c2370eb168) (dated 2025-03-21)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.10.05.05.o01 | SafeSearch | Preference selector | Strict; Moderate; Off | — | Controls whether Bing identifies sensitive content before it appears in web previews. With Strict, Bing respects the device setting regardless of the Bing.com setting. With Moderate or Off, Bing defaults to the Bing SafeSearch setting on Bing.com. | — |
| WB.10.05.05.o02 | Cloud content search | Settings section |  | — | Contains account options that control whether Windows Search shows results from personal or work/school Microsoft services. The Microsoft account option also enables Bing personalization for web search and suggestions when turned on. | — |
| WB.10.05.05.o03 | Microsoft account | Toggle | On; Off | — | When On, Windows Search shows results from personal OneDrive, Outlook, and other Microsoft services; Bing personalizes web results and suggestions. | For web search history associated with a Microsoft account, the page says to sign in to Windows with that account and turn this On. |
| WB.10.05.05.o04 | Work or School account | Toggle | On; Off | — | When On, Windows Search shows results from OneDrive for Business, Outlook, SharePoint, and more from Microsoft Search. | — |
| WB.10.05.05.o05 | History | Settings section |  | — | Contains the control for clearing search history saved on the device. | — |
| WB.10.05.05.o06 | Clear device search history | Button |  | — | Clears the search history saved on the device. | — |
| WB.10.05.05.o07 | Microsoft privacy dashboard | Link |  | — | Opens the place to view and delete search history associated with a Microsoft account. | — |
| WB.10.05.05.o08 | Bing Search History page | Link |  | — | Lets you include or exclude Microsoft-account search history from search suggestions. The page also identifies it as the place to view work or school account Microsoft Search history. | Sign in using the relevant Microsoft account or work or school account. |
| WB.10.05.05.o09 | Searching Windows | Settings page | Search the contents of Documents, Pictures, and Music folders; search the entire PC; customize search locations to include or exclude specific folders. | Windows uses indexing by default. | Provides settings for searching files and other content. Searching the entire PC may affect battery life and CPU consumption. | — |
| WB.10.05.05.o10 | Advanced indexing options | Sub-page |  | Semantic indexing is enabled by default on Copilot+ PCs. | Provides options to disable indexing and searching for specific locations or file types. | The stated semantic-indexing default and options apply to Copilot+ PCs. |
| WB.10.05.05.o11 | Search indexing in Windows FAQ | Link |  | — | Provides more information about search indexing. | — |
| WB.10.05.05.o12 | Bing Help | Link |  | — | Provides more information about SafeSearch. | — |

- SKU / hardware / version conditions: For improved Windows Search on Copilot+ PCs, the page recommends plugging in the PC to complete initial search indexing. Search indexing status is available under Settings > Privacy & security > Searching Windows.; Semantic indexing is enabled by default on Copilot+ PCs. Supported document formats listed are .txt, .pdf, .docx, .doc, .rtf, .pptx, .ppt, .xls, and .xlsx; supported image formats are .jpg/.jpeg, .png, .gif, .bmp, and .ico.; The page says data gathered from semantic indexing is stored locally on the PC and is not sent to Microsoft or used to train AI models.
- Windows 10 differences (legacy): Windows 10 path for SafeSearch, Cloud content search, and device search history: Start > Settings > Search > Permissions & History.; Windows 10 path for Searching Windows: Start > Settings > Search > Searching Windows.


### WB.10.06.01 — Privacy & security > Location

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Privacy & security > Location`

ms-settings URIs: `ms-settings:privacy-location`

Sources: [Official support article (T1)](https://support.microsoft.com/Windows/privacy/windows-location-service-and-privacy)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.10.06.01.o01 | Location services | Setting | On or Off | — | Controls whether the location service can be used on this device. When off, Windows and apps will not have access to device location. | Only an administrator sees this setting. |
| WB.10.06.01.o02 | Let apps access your location | Setting | On or Off | — | Controls location access for the current user account. When off, no apps are given location information. When on, apps have access to coarse location data, and you can choose which apps have access to precise location. | If the page says “Location services is off,” this setting cannot be turned on for an individual user account. |
| WB.10.06.01.o03 | App-specific precise location access | Per-app setting | On or Off for apps listed on the page | — | Lets you choose which apps have access to precise location. The page also shows when apps last accessed your location. | The page describes this control but does not provide a separate exact UI label for it. Apps that access certain Wi-Fi APIs may also appear on the page. |
| WB.10.06.01.o04 | Default location | Section with button and map/address controls | Select a point on the map or enter an address; select “Save” to set it. Clear the address box and select “Save” to remove it. | — | Sets a location Windows, apps, and services can use when a more exact location cannot be detected. | The page recommends this for devices without Wi-Fi and for desktop computers. |
| WB.10.06.01.o05 | Set default | Button |  | — | Opens the controls for setting a default location. | Appears under “Default location.” |
| WB.10.06.01.o06 | Allow location override | Setting | On or Off | — | Allows apps on a Cloud PC or virtual desktop to use the location of another device, such as the remote-connection client device. | Location services must be on for both the local and remote devices, and the app used to manage the remote connection must have location access. |
| WB.10.06.01.o07 | Notify when apps request location | Setting | On or Off | — | Controls prompts shown when apps request location while Location services is off. Turning it off stops these prompts. | Turning off prompts may affect apps that access Wi-Fi APIs; those apps may not work as expected, and notifications explaining why they are blocked will no longer appear. This does not affect app permission prompts when Location services is on. |

- Confirmations / errors: If the page says “Location services is off,” “Let apps access your location” cannot be turned on for an individual user account.; When Location services is off, a limited set of Windows features may still use the device IP address to provide contextual information for the area.; Windows will attempt to determine and share precise location during an emergency call regardless of location settings.
- SKU / hardware / version conditions: Only administrators see the “Location services” setting.; The “Default location” setting is highly recommended for devices that do not have Wi-Fi and for desktop computers.; “Allow location override” applies to Cloud PCs or virtual desktops.
- Windows 10 differences (legacy): The Windows 10 instructions describe a separate “Show location icon” control for Windows 10 Mobile and a “Location Notification” taskbar-icon setting for Windows 10 PCs.; The Windows 10 instructions for Microsoft Edge include “Allow access to location on this device,” “Allow apps to access your location,” and, for legacy Microsoft Edge, an app switch named “Microsoft Edge.”; The Windows 10 path is “Start > Settings > Privacy > Location.”; The Windows 10 user-level setting is named “Allow apps to access your location”; the page notes that in previous Windows versions it was called “Location service.”; Windows 10 PCs can add, remove, or move the “Location” tile through “Start > Settings > System > Notifications & actions > Edit your quick actions.”; Windows 10 device-wide location controls include “Change” and the “Location for this device” setting. The page says an administrator can switch it On or Off.; Windows 10 has a “Location history” section with a “Clear” control for clearing history on the device. The page states that local location-history storage and associated settings were removed in March 2025.; Windows 10 per-app controls are described under “Choose which apps can access your precise location.” The page also mentions “Allow desktop apps to access your location” where present.


### WB.10.06.02 — Privacy & security > Camera / Microphone

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Privacy & security > Microphone`; `Start > Settings > Privacy & security > Camera`

ms-settings URIs: `ms-settings:privacy-webcam`, `ms-settings:privacy-microphone`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/hardware/drivers/fix-microphone-problems)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.10.06.02.o01 | Microphone access | Toggle |  | — | Make sure it is turned on. | Windows 11 microphone access. |
| WB.10.06.02.o02 | Let apps access your microphone | Toggle |  | — | Turn it on, then choose which apps have access. | Windows 11 microphone app permissions. |
| WB.10.06.02.o03 | Let desktop apps access your microphone | Toggle |  | — | Turn it on to give desktop apps access. Desktop apps won't appear in the app list, and microphone access can't be turned off for individual desktop apps. | Windows 11 microphone permissions for desktop apps. |
| WB.10.06.02.o04 | Camera | Setting |  | — | Make sure it is turned on. | You might also need to allow camera access if your camera contains a microphone. |

- Windows 10 differences (legacy): Camera permissions path: Start > Settings > Privacy > Camera; make sure camera access for this device is on and “Allow apps to access your camera” is set to On.; Microphone permissions path: Start > Settings > Privacy > Microphone > Change; turn on “Allow apps to access your microphone.”; Windows 10 also names “Choose which Microsoft Store apps can access your microphone” and “Allow desktop apps to access your microphone”; the desktop-app switch should be set to On.


### WB.10.06.03 — Privacy & security > App permissions (general)

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Privacy & security`

ms-settings URIs: `ms-settings:privacy-appdiagnostics`, `ms-settings:privacy-broadfilesystemaccess`, `ms-settings:privacy-notifications`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/apps/app-permissions)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.10.06.03.o01 | App permission | Permission selection; specific control type not stated | A particular feature, for example, Location | — | Choose which apps can access the selected feature. | — |
| WB.10.06.03.o02 | Access all your files, peripheral devices, apps, programs, and registry | App permission |  | — | Can read or write files and registry settings; make changes to the computer and settings; use attached or built-in peripheral devices without notifying you; access location and platform features such as location history and app diagnostics. The app might not actually access these resources. Most of its permissions cannot be controlled in Settings > Privacy. | Windows Desktop apps fall into this category. |
| WB.10.06.03.o03 | Account Info | App permission |  | — | Access any of your account info. | — |
| WB.10.06.03.o04 | Allow elevation | App permission |  | — | Allows the app to run with administrator privileges without prompting the user first. | — |
| WB.10.06.03.o05 | App diagnostics | App permission |  | — | Get diagnostic information about other running apps. | — |
| WB.10.06.03.o06 | Bluetooth | App permission |  | — | Activate and use Bluetooth connections between your device and other devices. | — |
| WB.10.06.03.o07 | Calendar | App permission |  | — | Access your calendars. | — |
| WB.10.06.03.o08 | Call history | App permission |  | — | Access history of phone calls made on the device, in Skype or other telephony apps. | — |
| WB.10.06.03.o09 | Contacts | App permission |  | — | Access your contacts, people, or address book apps. | — |
| WB.10.06.03.o10 | Custom install actions | App permission |  | — | Install additional software. | — |
| WB.10.06.03.o11 | Email | App permission |  | — | Access your email and account info for your email accounts. | — |
| WB.10.06.03.o12 | Facial recognition | App permission |  | — | Activate and use facial recognition hardware. | — |
| WB.10.06.03.o13 | File system | App permission |  | — | Access files and folders available to you and read or write files, including documents, pictures, and music. | — |
| WB.10.06.03.o14 | Fingerprint reader | App permission |  | — | Activate and use fingerprint reader hardware. | — |
| WB.10.06.03.o15 | Local system services | App permission |  | — | Install a service on the machine that runs with maximum privileges. | — |
| WB.10.06.03.o16 | Location | App permission |  | — | Activate and use GPS or other location-finding features on the device; access location data in Maps and other location apps. | Given as an example of an App permission. |
| WB.10.06.03.o17 | Messaging | App permission |  | — | Access your instant messages and account info. | — |
| WB.10.06.03.o18 | Microphone | App permission |  | — | Activate and use the microphone on your device. | — |
| WB.10.06.03.o19 | Modifiable app | App permission |  | — | Enable the user to modify the app. | — |
| WB.10.06.03.o20 | Motion | App permission |  | — | Activate and use the accelerometer or other motion-sensing feature on your device. | — |
| WB.10.06.03.o21 | Music library | App permission |  | — | Access music files from the Music library on your device. | — |
| WB.10.06.03.o22 | Near field communications | App permission |  | — | Activate and use NFC connections between your device and other devices. | — |
| WB.10.06.03.o23 | Notifications | App permission |  | — | Access notifications found in action center. | — |
| WB.10.06.03.o24 | Packaged services | App permission |  | — | Install a service on the machine. | — |
| WB.10.06.03.o25 | Package write redirect compatability shim | App permission |  | — | Allows the app to create, modify, or delete files in the app’s install folder. | — |
| WB.10.06.03.o26 | Pictures library | App permission |  | — | Access picture files from the Pictures library on your device. | — |
| WB.10.06.03.o27 | Tasks | App permission |  | — | Access your task list in Outlook and other task-tracking apps. | — |
| WB.10.06.03.o28 | Unvirtualized resources | App permission |  | — | Write registry entries and files that are not cleaned up on uninstall. | — |
| WB.10.06.03.o29 | Video library | App permission |  | — | Access video files from the Video library on your device. | — |
| WB.10.06.03.o30 | Voice recognition | App permission |  | — | Activate and use voice recognition hardware. | — |
| WB.10.06.03.o31 | Webcam | App permission |  | — | Activate and use the camera on your device. | — |
| WB.10.06.03.o32 | WiFi | App permission |  | — | Activate and use WiFi connections between your device, the internet, and other devices. | — |
| WB.10.06.03.o33 | Wired connections | App permission |  | — | Activate and use wired connections, including Ethernet, USB, and Serial communications, between your device, the internet, and other devices. | — |

- Confirmations / errors: An app's ability to access resources does not mean it actually does so; the page recommends reviewing the developer's privacy policy for information about what the app collects or uses.; The Privacy page does not list apps that have permission to use all system resources, and Privacy settings cannot control what capabilities those apps can use.
- SKU / hardware / version conditions: Students using Windows 11 SE cannot download apps from Microsoft Store; apps are managed by their IT Admin.
- Windows 10 differences (legacy): Windows 10 navigation is Start > Settings > Privacy.; Windows 10 says to select an app (for example, Calendar) and choose which app permissions are on or off.


### WB.10.07 — Privacy & security > Smart App Control

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Windows Security app > settings (the article does not give a more specific path to Smart App Control).`; `Dans l’application Sécurité Windows, sélectionnez « Contrôle d’application & navigateur » > « paramètres Smart App Control ».`; `Raccourci indiqué : « Paramètres du contrôle d’application intelligent ».`; `Aucun chemin « Confidentialité et sécurité » ni aucune commande n’est indiqué.`; `Windows Security app > App & browser control`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/security/threat-malware-protection/smart-app-control-frequently-asked-questions); [Official support article (T1)](https://support.microsoft.com/fr-fr/windows/contr%C3%B4le-de-navigateur-d-application-dans-l-application-s%C3%A9curit%C3%A9-windows-8f68fb65-ebb4-3cfb-4bd7-ef0f376f3dc3); [Official support article (T1)](https://support.microsoft.com/en-us/windows/security/windows-security/app-browser-control-in-the-windows-security-app)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.10.07.o01 | Smart App Control | Setting; the article does not specify the control type or exact on-screen toggle label. | On; Off | Not stated as a settings default. The article says it automatically turns on in most cases, but may be turned off automatically for some users. | Can be turned on manually if available for the device, and can be turned off. Recent Windows updates allow enabling it again without a clean installation. | Availability and ability to turn it on may depend on device state and configuration. The article lists enterprise management, developer mode, Windows S mode, and optional diagnostic data being turned off as reasons it may be off. |
| WB.10.07.o02 | Send optional diagnostic data | Setup choice; not described as a Smart App Control page control. | Selected or not selected (the article only names the choice). | Not stated. | The article says optional diagnostic data must be enabled during setup to turn on Smart App Control in the case where it was turned off; otherwise, reset the PC or reinstall Windows. | Applies to the stated diagnostic-data reason Smart App Control may be off. |
| WB.10.07.o03 | How does Smart App Control work? | FAQ section; not a setting. |  | — | Checks whether its cloud-powered security service can confidently predict an app is safe. It allows apps predicted safe, blocks apps deemed malicious or potentially unwanted, and uses signature validity when it cannot make a confident prediction. | — |
| WB.10.07.o04 | Why is Smart App Control on? | FAQ section; not a setting. |  | — | Usually turns on automatically; it may turn off automatically for users whose regular tasks could be interrupted by it. | — |
| WB.10.07.o05 | Can I turn Smart App Control on manually? | FAQ section; not a setting. |  | — | Yes, if available for the device, through Windows Security app settings. Recent updates allow enabling it without a clean installation. | Must be available for the device. |
| WB.10.07.o06 | Why is Smart App Control turned off? | FAQ section; not a setting. |  | — | The article lists enterprise management or developer mode, a manual turn-off by a signed-in user, Windows S mode, and optional diagnostic data being off as possible reasons. | — |
| WB.10.07.o07 | Can I disable Smart App Control? | FAQ section; not a setting. |  | — | It can be disabled in Windows Security app settings. Recent updates allow re-enabling it without a clean installation. | — |
| WB.10.07.o08 | Does Smart App Control replace my antivirus software? | FAQ section; not a setting. |  | — | No. It works alongside Microsoft Defender or non-Microsoft antivirus tools. | — |
| WB.10.07.o09 | How can I tell Smart App Control to let this one specific app through? | FAQ section; not a setting. |  | — | There is no way to bypass protection for an individual app. The article suggests turning Smart App Control off or asking the app developer to sign the app with a valid signature. | — |
| WB.10.07.o10 | What does untrusted mean? | FAQ section; not a setting. |  | — | An app is considered untrusted when the service cannot confidently predict whether it is safe and the app does not have a valid signature. | — |
| WB.10.07.o11 | I'm an app developer, how can I get Smart App Control to not block my app? | FAQ section; not a setting. |  | — | The article says to sign the app with a valid certificate. | — |
| WB.10.07.o12 | How can I provide feedback about Smart App Control? | FAQ section; not a setting. |  | — | The article directs users to Feedback Hub, then to choose the Security and Privacy category and Smart App Control. | — |

- Confirmations / errors: After evaluation is complete, or after Smart App Control is manually switched on or off, you cannot return to Evaluation mode unless you reinstall or reset Windows.; For an overview, the article points to “App & Browser Control” in the Windows Security app.; If optional diagnostic data is off, the article says to reset the PC or reinstall Windows and select “Send optional diagnostic data” during setup.; The article does not provide exact Smart App Control toggle labels, detailed steps, or a more specific Windows Security navigation path.; The article states that an app-specific bypass is unavailable.; The page states that Smart App Control works alongside Microsoft Defender or non-Microsoft antivirus tools.
- Restart / sign-out: For Windows S mode, the article says to turn S mode off and then reset the PC.; For optional diagnostic data being off, the article says to reset the PC or reinstall Windows; it does not specify a sign-out procedure.; Recent Windows updates allow Smart App Control to be enabled or re-enabled without a clean installation.
- Keyboard shortcuts: App & browser control; Check for updates; Smart App Control Frequently Asked Questions; Smart App Control settings; « Paramètres du contrôle d’application intelligent ».
- SKU / hardware / version conditions: La page recommande d’installer les dernières mises à jour Windows et Defender pour une expérience optimale.; Smart App Control ne peut être utilisé que sur les nouvelles installations de Windows 11. Pour un appareil déjà en cours d’exécution ayant reçu la fonctionnalité par une mise à jour Windows, une réinitialisation du PC ou une réinstallation de Windows est nécessaire pour l’activer.; The article applies to Windows 11.; The article lists enterprise-managed devices, devices configured for developer mode, and devices running Windows in S mode as possible reasons Smart App Control may be off.; The article says availability is required to turn it on manually.
- Windows 10 differences (legacy): Smart App Control is not available in Windows 10.; Smart App Control n’est pas disponible dans Windows 10.


## WB.11 — Windows Update


### WB.11.01 — Windows Update

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Windows Update`; `Start > Settings > Windows Update > Pause updates`; `Start > Settings > Windows Update > Schedule the restart`; `Start > Settings > Windows Update > Update history`; `Start > Settings > Windows Update > Update history > Uninstall updates`

ms-settings URIs: `ms-settings:windowsupdate`, `ms-settings:windowsupdate-action`

Sources: [Official support article (T1)](https://support.microsoft.com/windows/update-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.11.01.o01 | Check for updates | Button |  | — | Manually checks for updates. | — |
| WB.11.01.o02 | Pause updates | Setting with a selection control | Select the pause length you prefer; specific choices are not stated. | — | Temporarily pauses updates from being downloaded and installed. | After the pause limit is reached, install the latest updates before pausing again. |
| WB.11.01.o03 | Schedule the restart | Setting | Choose a convenient time. | — | Schedules a restart to finish installing an update. | An update must be installed. |
| WB.11.01.o04 | active hours | Setting |  | — | Can be set so automatic restarts for updates happen only when you are not using the device. | — |
| WB.11.01.o05 | Update history | Sub-page |  | — | Shows which updates were installed and when; provides access to remove specific updates. | — |
| WB.11.01.o06 | Uninstall updates | Sub-page |  | — | Lets you select an installed update and select **Uninstall**. | Not all updates can be removed. |
| WB.11.01.o07 | Keep your PC up to date with active hours | Link |  | — | Provides information about using active hours. | — |
| WB.11.01.o08 | Learn how to temporarily pause updates | Link |  | — | Provides information about temporarily pausing updates. | — |
| WB.11.01.o09 | Find out how to schedule a restart to install updates at a convenient time | Link |  | — | Provides information about scheduling a restart. | — |
| WB.11.01.o10 | Metered connections in Windows | Link |  | — | Provides information about metered connections; some Windows updates might not install automatically on a metered connection. | Applies when using a metered connection. |

- Confirmations / errors: Not all updates can be removed.; To finish installing an update, the device needs to restart. Windows will try to restart when the device is not in use; if it cannot, it will ask you to schedule the restart.; Updates cannot be stopped entirely; they will eventually need to be downloaded and installed.; Windows recommends keeping laptops and tablets plugged in when they are not in use so they have enough power to complete updates.
- Restart / sign-out: A restart is required to finish installing an update.; Windows tries to restart the device when it is not in use and may ask you to schedule the restart if it cannot.
- SKU / hardware / version conditions: Automatic feature updates are offered only to consumer devices and non-managed business devices running Windows 10 and Windows 11.
- Windows 10 differences (legacy): The page states that the last feature update for Windows 10 is version 22H2.; Windows 10 offers **Pause updates for 7 days** and, in Advanced options under Pause updates, a dropdown to specify a date for updates to resume. This is available in Windows 10 version 1909 or later, or version 1809 for Pro or Enterprise.; Windows 10 uses **View update history** to reach update history and then **Uninstall updates**.; Windows 10's update path is Start > Settings > Update & Security > Windows Update.


### WB.11.01.01 — Windows Update > Pause updates

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Windows Update`; `Start > Settings > Windows Update > Pause updates`

Sources: [Official support article (T1)](https://support.microsoft.com/en-gb/windows/pause-updates-in-windows-643e9ea7-3cf6-7da6-a25c-95d4f7f099fe); [Official support article (T1)](https://support.microsoft.com/en-us/windows/deployment/updates-lifecycle/windows-update-faq); [Official support article (T1)](https://support.microsoft.com/en-us/ph/6527)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.11.01.01.o01 | Amount of time you'd like to pause the update for | Not specified | Not specified | — | Pauses the update from being downloaded and installed for the selected amount of time. | — |
| WB.11.01.01.o02 | Schedule the restart | Selection | Choose a convenient time | — | Schedules a restart to finish installing updates. | Available if you're asked to restart while busy using the device. |
| WB.11.01.01.o03 | Keep your PC up to date with active hours | Link |  | — | Leads to information about setting active hours so the device restarts for updates when you're not using the PC. | — |
| WB.11.01.01.o04 | Pause updates | Setting/page option; select it, then select a pause length | The pause lengths are not specified on this page. | — | Temporarily pauses updates from being downloaded and installed. Updates cannot be stopped entirely; after the pause limit is reached, the latest updates must be installed before updates can be paused again. | — |
| WB.11.01.01.o05 | Learn how to temporarily pause updates | Link |  | — | Links to information about temporarily pausing updates. | — |

- Confirmations / errors: After the pause time limit is reached, you must install the latest updates.; You must restart your device to finish installing the updates.
- Restart / sign-out: Schedule the restart to choose a more convenient time.; Set active hours to help ensure the device restarts for updates when you're not using your PC.
- Windows 10 differences (legacy): For Windows 10, version 1903 or later, Windows 10 Pro, version 1809, or Windows 10 Enterprise, version 1809, the page offers “Pause updates for 7 days” or “Advanced options”; under “Pause updates,” use the dropdown menu to specify a date for updates to resume.; Windows 10 November 2019 Update (version 1909) or later: the page says to select “Pause updates for 7 days”; under “Advanced options,” the “Pause updates” drop-down lets you specify a date for updates to resume.; Windows 10 instructions say to select “Pause updates for 7 days.” They also describe selecting “Advanced options” and using the “Pause updates” drop-down to specify a date for updates to resume. These Windows 10 controls and choices are not stated for Windows 11.; Windows 10 navigation is Start > Settings > Update & Security > Windows Update.; Windows 10 support will end on October 14, 2025. After that date, Microsoft will no longer provide free software updates from Windows Update, technical assistance, or security fixes for Windows 10.; Windows version 1809: the page limits the described pause instructions to Pro or Enterprise editions.


### WB.11.02 — Windows Update > Update history

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Windows Update > Update history`; `Start > Settings > Windows Update > Update history > Uninstall updates`; `Windows 11: Start > Settings > Windows Update > Update history`; `Windows 11 removal path: Start > Settings > Windows Update > Update history > Uninstall updates`; `Windows 11: Start > Settings > Windows Update > More options > Update history`

ms-settings URIs: `ms-settings:windowsupdate-history`

Sources: [Official support article (T1)](https://support.microsoft.com/help/12373); [Official support article (T1)](https://support.microsoft.com/en-us/windows/deployment/updates-lifecycle/windows-update-faq); [Official support article (T1)](https://support.microsoft.com/windows/uninstall-a-windows-update-c77b8f9b-e4dc-4e9f-a803-fdec12e59fb0)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.11.02.o01 | Update history | Settings page |  | — | Shows which updates were installed and when. The list can also be used to remove specific updates, although Microsoft does not recommend doing so unless necessary. | — |
| WB.11.02.o02 | Uninstall updates | Link or button |  | — | Opens the uninstall-updates area from Update history. | Not all updates can be removed. |
| WB.11.02.o03 | Uninstall | Button |  | — | Uninstalls the selected update. | Select an update first; not all updates can be removed. |
| WB.11.02.o04 | View update history | Button or link label shown on the page | Not stated | — | Opens the update history. | — |
| WB.11.02.o05 | Select the update you want to remove | Update selection | Updates available for removal; specific choices are not listed | — | Selects an update to remove. | The page says not all updates can be removed. |

- Confirmations / errors: Before uninstalling a security update, understand the risks. For Windows 11, try to reinstall your version of Windows first.; Microsoft does not recommend removing installed updates unless necessary.; No confirmation prompt or error message is stated.; Not all updates can be removed.; Some updates cannot be uninstalled.
- Windows 10 differences (legacy): Windows 10 path: Start > Settings > Update & Security > Windows Update > View update history > Uninstall updates. The Windows 11 path uses Update history.; Windows 10 path: Start > Settings > Update & Security > Windows Update > View your update history > Uninstall updates.; Windows 10 uses the path Start > Settings > Update & Security > Windows Update > View update history > Uninstall updates. The page labels the history entry “View update history.”


### WB.11.03 — Windows Update > Advanced options

Status: PARTIAL. Official source confirms the page exists but documents few or none of its controls.

Navigation paths (as stated in sources): `Start > Settings > Windows Update > Advanced options`; `Start > Settings > Windows Update > Advanced options > Active hours`

ms-settings URIs: `ms-settings:windowsupdate-options`, `ms-settings:windowsupdate-activehours`, `ms-settings:windowsupdate-optionalupdates`, `ms-settings:windowsupdate-restartoptions`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/Windows/Deployment/Updates-Lifecycle/keep-your-pc-up-to-date-with-active-hours); [Official support article (T1)](https://support.microsoft.com/en-us/windows/keep-your-pc-up-to-date-with-active-hours-de79813c-7919-5fed-080f-0871c7bd9bde)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.11.03.o01 | Active hours | Setting or navigation item |  | — | Select it to configure active hours. | — |
| WB.11.03.o02 | Adjust active hours | Dropdown | Automatically; Manually | — | Automatically adjusts active hours based on your device's activity. Manually lets you select your own active hours. | Available after selecting Active hours. |

- Windows 10 differences (legacy): Automatic option: “Automatically adjust active hours for this device based on activity”; turn it on, then select “Save”.; Manual option: select “Change” next to your current active hours, choose the start and end times, then select “Save”.; Navigation: Start > Settings > Update & Security > Windows Update > Change active hours.; The page states that Windows 10 support ended on October 14, 2025.; To set Windows 10 active hours manually, select Change next to your current active hours, choose the start and end times, then select Save.; Windows 10 navigation is Start > Settings > Update & Security > Windows Update > Change active hours.; Windows 10 offers the toggle “Automatically adjust active hours for this device based on activity” and a Save button.


### WB.11.04 — Windows Update > Windows Insider Program

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Windows Update > Windows Insider Program`

ms-settings URIs: `ms-settings:windowsinsider`, `ms-settings:windowsinsider-optin`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/join-the-windows-insider-program-and-manage-insider-settings-ef20bb3d-40f4-20cc-ba3c-a72c844b563c)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.11.04.o01 | Get Started | button |  | — | Starts setup to receive Insider builds. | After joining the Insider program. |
| WB.11.04.o02 | Link an account | button | The account used to register with the Insider program, or a different account. | — | Links an account for the Insider program. | — |
| WB.11.04.o03 | Use a different account | option | Select the type of account used and enter those credentials. | — | Lets you use an account other than the one used to register with the Insider program. | Available when using a different account. |
| WB.11.04.o04 | Insider channel | selection | Select the channel to receive builds for; channel names are not stated. | — | Select a channel, then select **Continue**. | — |
| WB.11.04.o05 | Continue | button |  | — | Proceeds after selecting an Insider channel and after reviewing the device agreements. | — |
| WB.11.04.o06 | Restart now | button |  | — | Restarts the device and starts receiving the selected builds right away. | Alternatively, restart later to start receiving builds then. |
| WB.11.04.o07 | Stop getting preview builds | option |  | — | Starts the instructions to opt out of Insider builds. | The page recommends opting out when the device is using a production build. |

- Confirmations / errors: Review the agreements for your device, then select **Continue**.
- Restart / sign-out: Select **Restart now** to restart immediately, or restart the device later to start receiving the selected builds.; To opt out, select **Stop getting preview builds** and follow the instructions.
- Windows 10 differences (legacy): After selecting the Insider channel and reviewing the device agreements, the button is **Confirm** rather than **Continue**.; Settings path: Start > Settings > Update & Security > Windows Insider Program.; To opt out, change the **Stop getting preview builds** option to **On**.


### WB.11.06 — Windows Update > Delivery Optimization

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > Settings > Windows Update > Advanced options > Delivery Optimization`; `Start > Settings > Windows Update > Advanced options > Delivery Optimization > Activity monitor`

ms-settings URIs: `ms-settings:delivery-optimization`, `ms-settings:delivery-optimization-activity`, `ms-settings:delivery-optimization-advanced`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/deployment/updates-lifecycle/delivery-optimization-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.11.06.o01 | Go to Delivery Optimization settings | Link |  | — | Takes you to Delivery Optimization settings. | — |
| WB.11.06.o02 | Allow downloads from other devices | Toggle | On or off. When on: Devices on my local network; Devices on the internet and my local network. | — | Controls whether your PC can get downloads from other PCs and share parts of downloads with other PCs. | When on, you can choose which types of devices to allow downloads from. The option may be grayed out if your device is managed by your organization. |
| WB.11.06.o03 | Devices on my local network | Choice | Devices on my local network; Devices on the internet and my local network | — | Allows downloads from devices on your local network. | Available when Allow downloads from other devices is on. |
| WB.11.06.o04 | Devices on the internet and my local network | Choice | Devices on my local network; Devices on the internet and my local network | — | Allows downloads from devices on the internet and your local network. | Available when Allow downloads from other devices is on. |
| WB.11.06.o05 | Activity monitor | Sub-page |  | — | Shows a breakdown of downloads by source, average speed, and upload stats for the current month, including the boost from other PCs on the internet. | — |
| WB.11.06.o06 | Limit bandwidth used for downloads | Setting | Absolute bandwidth limit; Percentage of measured bandwidth⁠:⁠ | — | Limits bandwidth for Delivery Optimization downloads. For either choice, you can set limits for background downloads, foreground downloads, or both. Percentage limits use less bandwidth at lower values but deliver updates more slowly. | Bandwidth limits apply to automatic updates downloads, not downloads you manually initiate. |
| WB.11.06.o07 | Absolute bandwidth limit | Choice and value control | Background, foreground, or both; maximum bandwidth entered in Mbps. | — | Sets the maximum bandwidth used for downloads. | Available under Download options when limiting download bandwidth. |
| WB.11.06.o08 | Percentage of measured bandwidth⁠:⁠ | Choice and percentage control | Background, foreground, or both; percentage of available bandwidth. | — | Limits downloads to a percentage of available bandwidth. Lower values use less bandwidth but cause updates to be delivered more slowly. | Available under Download options when limiting download bandwidth. |
| WB.11.06.o09 | Limit how much bandwidth is used for uploading updates to other devices on the internet | Percentage selection | A percentage for upload bandwidth. | — | Limits the percentage of bandwidth used for uploading updates. | Found under Upload options. |
| WB.11.06.o10 | Monthly upload limit | Selection | 1 to 500 GB | — | Sets the maximum amount of data allowed per month for uploads. The page says this can be used together with the upload bandwidth percentage limit. | Found under Upload options. |

- Confirmations / errors: If the option to turn Delivery Optimization off is unavailable, the device is managed by an organization and the IT administrator requires Delivery Optimization to be enabled. The page may show the note “Some of these settings are managed by your organization”; Allow downloads from other devices may be grayed out, as may other options depending on administrator settings.; Turning off Delivery Optimization does not disable updates; it makes each device download updates directly from Microsoft rather than from a local machine that previously downloaded them.
- Windows 10 differences (legacy): The Windows 10 managed-device note says “Some of these settings are hidden or managed by your organization”; it names “Allow downloads from other PCs” as the grayed-out option.; The page includes a Windows 10 cache-clearing procedure: search for disk cleanup, select the Disk Cleanup app, check “Delivery Optimization Files,” select “OK,” then select “Delete Files” in the dialog. The page does not give a Windows 11 Settings path for this procedure.; Windows 10 activity monitor path is Start > Settings > Update & security > Delivery Optimization > Activity Monitor.; Windows 10 download bandwidth choices are labeled “Absolute bandwidth” and “Percentage of measured bandwidth⁠:⁠”.; Windows 10 navigation is Start > Settings > Update & security > Delivery Optimization. Bandwidth controls are under Start > Settings > Update & security > Delivery Optimization > Advanced options.; Windows 10 network-sharing wording includes “PCs on my local network” and “Allow downloads from other PCs.” It says to choose “PCs on my local network” to get downloads from local-network PCs but not internet PCs, or turn off “Allow downloads from other PCs” to get downloads from neither local nor internet PCs.


## WB.12 — Beyond Settings (tools, legacy, management)


### WB.12.02 — Control Panel

Status: PARTIAL. Official source confirms the page exists but documents few or none of its controls.

Navigation paths (as stated in sources): `Search for Control Panel in the Start menu.`; `The page also says “Use the keyboard shortcut,” but does not state a shortcut.`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/windows/experience/system-configuration-tools-in-windows); [Official support article (T1)](https://support.microsoft.com/en-us/windows/experience/system-configuration-tools-in-windows); [Official support article (T1)](https://support.microsoft.com/en-au/windows/system-configuration-tools-in-windows-f8a49657-b038-43b8-82d3-28bea0c5666b)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.12.02.o01 | Control Panel | Windows feature with applets | The page gives examples of settings for system time and date, hardware, and network configurations, and says there are more. | — | Provides a centralized location to view and manipulate system settings and controls. Many settings are being migrated to the Settings app. The page recommends using Settings whenever possible; Control Panel remains for compatibility and to access settings that have not yet migrated. | — |

- Confirmations / errors: The page recommends using the Settings app whenever possible; Control Panel remains for compatibility and to access settings that have not yet migrated.; The page says many Control Panel settings are being migrated to the Settings app.


### WB.12.04 — Device Manager

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Right-click Start > Device Manager`; `Right-click the Start menu > Device Manager. (Given for opening Device Manager; no Settings path or command is stated.)`; `Right-click the Start menu > Device Manager`

Sources: [Official support article (T1)](https://support.microsoft.com/help/4028443/); [Official support article (T1)](https://support.microsoft.com/pt-br/windows/atualize-um-driver-de-hardware-que-n%C3%A3o-esteja-funcionando-adequadamente-5c4da477-59fb-355b-45d6-1aa742268c9b); [Official support article (T1)](https://support.microsoft.com/en-us/windows/update-drivers-through-device-manager-in-windows-ec62f46c-ff14-c91d-eead-d7126dc1f7b6)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.12.04.o01 | Reinstall a device driver | article section |  | — | Describes uninstalling a device and restarting Windows so Windows can attempt to reinstall its driver. | — |
| WB.12.04.o02 | Rollback the driver (if a recent update was installed) | article section |  | — | Describes reverting to a previous driver. | Administrator permissions are required. The article says reinstalling the previous driver often resolves an issue that occurred after a Windows Update. |
| WB.12.04.o03 | Update a device driver automatically | article section |  | — | Describes searching automatically for an updated driver and, if available, installing it. | — |
| WB.12.04.o04 | Update a device driver manually | article section |  | — | Describes selecting downloaded driver files from the computer. | Drivers must first be downloaded from the device manufacturer's website. |
| WB.12.04.o05 | Contact the device manufacturer | article section |  | — | Advises contacting the manufacturer for the latest compatible driver or additional support. | Applies if the driver cannot be updated using the article's steps. |
| WB.12.04.o06 | Device Manager | app |  | — | Used to uninstall, roll back, and update device drivers. | Open by right-clicking Start and selecting Device Manager. |
| WB.12.04.o07 | Display adapters | device category example |  | — | Example category to expand when working with a display adapter driver. | — |
| WB.12.04.o08 | Printers | device category example |  | — | Example category to expand when working with a printer driver. | — |
| WB.12.04.o09 | Uninstall device | context-menu command |  | — | Opens the Uninstall Device window. | Used to reinstall a device driver. |
| WB.12.04.o10 | Uninstall | button |  | — | Confirms uninstalling the device. | Appears in the Uninstall Device window. |
| WB.12.04.o11 | Properties | context-menu command |  | — | Opens the device's properties. | Used in the driver rollback steps. |
| WB.12.04.o12 | Driver | tab |  | — | Selected to access the driver rollback command. | Appears in the device Properties window. |
| WB.12.04.o13 | Roll Back Driver | command |  | — | Starts the driver rollback process. | The article says to choose a reason when prompted and then confirm. |
| WB.12.04.o14 | Update driver | context-menu command |  | — | Opens the Update Drivers window. | Used for automatic or manual driver updates. |
| WB.12.04.o15 | Search automatically for updated driver software | option |  | — | Searches for updated drivers and installs them if found. | Selected in the Update Drivers window. |
| WB.12.04.o16 | Search for updated drivers on Windows update | option |  | — | Provides a route to search for updated drivers through Windows Update. | Offered if Windows doesn't find updated drivers. |
| WB.12.04.o17 | Close | button |  | — | Closes the Update Drivers window. | — |
| WB.12.04.o18 | Browse my computer for drivers | option |  | — | Opens the flow for selecting downloaded driver files on the computer. | Selected for a manual driver update. |
| WB.12.04.o19 | Browse... | button |  | — | Opens the Browse For Folder window to locate downloaded drivers. | — |
| WB.12.04.o20 | OK | button |  | — | Confirms the selected driver folder. | — |
| WB.12.04.o21 | Next | button |  | — | Continues the manual driver update after the driver path is checked. | — |
| WB.12.04.o22 | Yes | button |  | — | Confirms the driver rollback. | Shown after choosing a reason for the rollback. |
| WB.12.04.o23 | Shut down or sign out | Start menu command |  | — | Opens the menu containing Restart. | Used in the reinstall steps. |
| WB.12.04.o24 | Restart | command |  | — | Restarts the Windows device. | After uninstalling a device, Windows attempts to reinstall its driver when the device restarts. The article also says to restart after rollback if prompted. |
| WB.12.04.o25 | Shut down or sign out > Restart | Start menu command path | Not stated | — | Restarts the Windows device. | The article instructs restarting after uninstalling a device driver. |
| WB.12.04.o26 | Reinstall a device driver — Device Manager | Context-menu command and confirmation button | Right-click the device > Uninstall device; in the Uninstall Device window, select Uninstall. | — | Uninstalls the selected device. After restarting Windows, Windows attempts to reinstall its driver. | Expand the relevant device category and identify the device first. Some devices, such as printers, might need to be reinstalled manually. |
| WB.12.04.o27 | Device category arrow | Expand/collapse control | Select the arrow next to the desired device category; examples given are Display adapters and Printers. | — | Expands the category to show devices. | — |
| WB.12.04.o28 | Rollback the driver (if a recent update was installed) — Properties | Context-menu command and tab | Right-click the device > Properties > Driver tab > Roll Back Driver. | — | Starts rolling back the driver; choose a reason when prompted, then select Yes to confirm. | Administrator permissions are required. If the issue occurred after a Windows Update, reinstalling the previous driver often resolves it. |
| WB.12.04.o29 | Update driver — automatic | Context-menu command | Right-click the device > Update driver > Search automatically for updated driver software. | — | Windows searches for updated drivers and installs them if found. | Expand the relevant device category and identify the device first. |
| WB.12.04.o30 | Update driver — manual | Context-menu command | Right-click the device > Update driver > Browse my computer for drivers > Browse... > choose the driver folder > OK > Next. | — | Windows installs the driver if it finds an updated version. | First download the drivers from the device manufacturer's website, matching the Windows version and architecture; uncompress them if necessary. |
| WB.12.04.o31 | Windows Update | Update service | No specific control choices stated. | — | The page says Windows Update is the best way to get driver updates automatically and that it downloads and installs drivers for many hardware devices. | Drivers updated through Windows Update are normally recent and up to date. |

- Confirmations / errors: During manual updating, Windows installs the drivers if it finds they are updated versions.; For rollback, choose a reason when prompted and select Yes to confirm.; If Windows does not find updated drivers, the article suggests searching through Windows Update.; If Windows doesn't find updated drivers, the article offers the option Search for updated drivers on Windows update.; If Windows finds no updated driver, it may offer Search for updated drivers on Windows update.; If Windows finds updated drivers during an automatic search, it installs them.; If a manual search finds no newer version, the message is: “The best drivers for your device are already installed.”; If updated drivers are already installed during the manual flow, Windows displays: “The best drivers for your device are already installed.”; If updated drivers are already installed, the message is: "The best drivers for your device are already installed".; The Uninstall Device window asks for confirmation; select Uninstall.; The Uninstall Device window asks the user to confirm uninstalling the device by selecting Uninstall.; The article’s sections are Reinstall a device driver; Rollback the driver (if a recent update was installed); Update a device driver automatically; Update a device driver manually; Contact the device manufacturer.
- Restart / sign-out: After restarting, Windows attempts to reinstall the driver. The article says some devices, such as printers, might need to be reinstalled manually.; After rollback, restart the device if prompted.; After uninstalling a device, restart using Start > Shut down or sign out > Restart; Windows then attempts to reinstall the driver.; After uninstalling a device, right-click Start > Shut down or sign out > Restart. Windows then attempts to reinstall the driver.; For driver reinstallation: right-click Start > Shut down or sign out > Restart.; For rollback: restart the device if prompted.
- SKU / hardware / version conditions: Applies to Windows 11 and Windows 10.; For a manual update, download the correct drivers for the Windows version and architecture from the device manufacturer's website; uncompress them if necessary.; For manual updating, download drivers that match the Windows version and architecture.; Manually downloaded drivers should match the Windows version and architecture.; Rollback requires administrator permissions.; Some devices, such as printers, might need manual reinstallation.; Some devices, such as printers, might need to be reinstalled manually.; The article advises avoiding driver downloads from sites other than the manufacturer's official site.; The article applies to Windows 11 and Windows 10.; The page advises downloading drivers only from the device manufacturer's official site.
- Windows 10 differences (legacy): The page says it applies to both Windows 11 and Windows 10; it does not state any differences between them.


### WB.12.05 — Task Manager

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `In the Windows search bar, type "Task manager" and press Enter.`; `Press Ctrl+Shift+Esc to open Task Manager directly.`; `Right-click Start > Task Manager`; `Ctrl+Shift+Esc`; `Press CTRL+SHIFT+ESC`

Sources: [Official support article (T1)](https://support.microsoft.com/en-us/accessibility/windows/use-a-screen-reader-to-navigate-windows-support-tools); [Official support article (T1)](https://support.microsoft.com/en-au/windows/system-configuration-tools-in-windows-f8a49657-b038-43b8-82d3-28bea0c5666b); [Official support article (T1)](https://support.microsoft.com/en-us/windows/experience/system-configuration-tools-in-windows)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.12.05.o01 | Processes | Task Manager tab | Apps, background processes, and Windows processes are categorized; CPU, memory, disk, and network usage are displayed. | — | Shows active processes and resource usage. The End Task feature can terminate unresponsive or resource-hogging processes. | — |
| WB.12.05.o02 | End Task | Feature in Processes |  | — | Terminates unresponsive or resource-hogging processes. | — |
| WB.12.05.o03 | Performance | Task Manager tab | CPU, memory, disk activity, Wi-Fi, and GPU performance. | — | Provides a graphical, real-time overview of system resources. | — |
| WB.12.05.o04 | App History | Task Manager tab | CPU time and network activity. | — | Displays resource usage for installed applications over time. | — |
| WB.12.05.o05 | Startup apps | Task Manager tab | Enabled or disabled for startup. | — | Manages applications that launch automatically during Windows startup and indicates their startup status. | — |
| WB.12.05.o06 | Users | Task Manager tab |  | — | Displays users currently signed into the computer and their resource consumption. | — |
| WB.12.05.o07 | Details | Task Manager tab |  | — | Provides advanced, detailed information about each process currently running. | — |
| WB.12.05.o08 | Services | Task Manager tab | Running or Stopped. | — | Lists Windows services and provides options to start, stop, or restart them. | — |
| WB.12.05.o09 | Settings | Task Manager tab | Appearance and settings; General settings; Advanced options. | — | Customizes the appearance and functionality of Task Manager. | — |
| WB.12.05.o10 | Appearance and settings | Settings option | Always-on-top; real-time updates. | — | Customizes the look and behaviour of Task Manager, including always-on-top and real-time updates. | — |
| WB.12.05.o11 | General settings | Settings option | Default start page and other key preferences. | — | Sets the default start page and other key preferences. | — |
| WB.12.05.o12 | Advanced options | Settings option | Live kernel memory dumps and other in-depth features. | — | Manages live kernel memory dumps and other in-depth features. | — |
| WB.12.05.o13 | CPU load | Not specified | Not specified | — | Task Manager lets you assess CPU load. | — |
| WB.12.05.o14 | memory usage | Not specified | Not specified | — | Task Manager lets you assess memory usage. | — |
| WB.12.05.o15 | disk activity | Not specified | Not specified | — | Task Manager lets you assess disk activity. | — |
| WB.12.05.o16 | network usage | Not specified | Not specified | — | Task Manager lets you assess network usage. | — |
| WB.12.05.o17 | running applications | Not specified | Not specified | — | Task Manager lets you view and manage running applications. | — |
| WB.12.05.o18 | unresponsive programs | Not specified | Not specified | — | Task Manager lets you terminate unresponsive programs. | — |
| WB.12.05.o19 | startup applications | Not specified | Not specified | — | Task Manager lets you adjust startup applications. | — |
| WB.12.05.o20 | active user sessions | Not specified | Not specified | — | Task Manager lets you monitor active user sessions. | — |
| WB.12.05.o21 | Task Manager | Application; system monitor and startup manager |  | — | View and manage running applications; assess CPU load, memory usage, disk activity, and network usage; terminate unresponsive programs; adjust startup applications; monitor active user sessions. | — |

- Keyboard shortcuts: CTRL+SHIFT+ESC; Ctrl+Shift+Esc opens Task Manager directly.; Ctrl+Shift+Esc opens Task Manager.
- Windows 10 differences (legacy): The Windows 10 section calls the tab "Startup"; the Windows 11 section calls it "Startup apps".; The Windows 10 section lists Processes, Performance, App History, Startup, Users, Details, and Services, but does not list the Settings tab or its options.; The Windows 10 section says to select the "More Details" button and press Spacebar to view the detailed breakdown of the primary tabs and features.


### WB.12.06 — Disk Management

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Right-click (or long-press) the Start button > Disk Management`; `Search > type “Disk Management” > Create and format hard disk partitions`; `Windows + R > type “diskmgmt.msc” > OK`; `Computer Management > Storage > Disk Management`; `Start menu: Right-click (or long-press) the Start button, then select Disk Management.`

Sources: [Official support article (T1)](https://support.microsoft.com/en-gb/windows/disk-management-in-windows-ad88ba19-f0d3-0809-7889-830f63e94405); [Official support article (T1)](https://support.microsoft.com/en-US/Windows/Experience/Storage-FileManagement/disk-management-in-windows); [Official documentation (T1)](https://learn.microsoft.com/en-us/windows-server/storage/disk-management/overview-of-disk-management) (dated 2025-06-26)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.12.06.o01 | Summary | Page section |  | — | Describes Disk Management as a built-in Windows tool for managing disks and volumes. | — |
| WB.12.06.o02 | Initialize a new disk | Page section |  | — | Provides steps for initializing a disk. | — |
| WB.12.06.o03 | Initialize Disk | Command |  | — | Opens the Initialize Disk dialog box for the selected disk. | Select the disk first. If it is Offline, select Online first. |
| WB.12.06.o04 | Online | Command |  | — | Brings an Offline disk online. | Use when the disk is listed as Offline. |
| WB.12.06.o05 | GPT | Partition-style option |  | The page says you can accept the default partition style or change it, but does not identify which style is the default. | Recommended for a modern system with UEFI firmware and for support of large drives and more than four partitions. | — |
| WB.12.06.o06 | MBR | Partition-style option |  | The page says you can accept the default partition style or change it, but does not identify which style is the default. | Use when working with older hardware or operating systems that do not support UEFI. | — |
| WB.12.06.o07 | OK | Command |  | — | Accepts the selected partition style in the Initialize Disk dialog box. | — |
| WB.12.06.o08 | Create and format a new partition (volume) | Page section |  | — | Provides steps for creating and formatting a new partition or volume. | Requires administrator login and unallocated disk space or free space within an extended partition. |
| WB.12.06.o09 | New Simple Volume | Command |  | — | Starts the New Simple Volume Wizard. | Right-click an unallocated region on the hard disk. |
| WB.12.06.o10 | Next | Command |  | — | Advances through the New Simple Volume Wizard. | — |
| WB.12.06.o11 | Volume size | Entry | Enter a size in megabytes (MB), or use the maximum default size. | Maximum default size. | Sets the size of the volume to create. | — |
| WB.12.06.o12 | Drive letter | Selection | Accept the default drive letter or choose a different drive letter. | Default drive letter. | Identifies the partition. | — |
| WB.12.06.o13 | Format Partition | Wizard dialog | Do not format this volume; format the volume with the default settings. | The page says to select Next to format with the default settings, but does not specify the settings. | Lets you defer formatting or format the volume with default settings. | — |
| WB.12.06.o14 | Do not format this volume | Option |  | — | Skips formatting the volume at this point in the wizard. | — |
| WB.12.06.o15 | Finish | Command |  | — | Completes creation of the volume after reviewing the choices. | — |
| WB.12.06.o16 | To format an existing partition (volume) | Page section |  | — | Provides steps for formatting an existing volume. | — |
| WB.12.06.o17 | Format | Command |  | — | Opens the Format dialog box for the selected volume. | Right-click the volume to format. |
| WB.12.06.o18 | Perform a quick format | Option |  | — | A quick format creates a new file table but does not fully overwrite or erase the volume; it is much faster than a normal format. | — |
| WB.12.06.o19 | Extend a basic volume | Page section |  | — | Links to instructions for extending a basic volume. | — |
| WB.12.06.o20 | Shrink a basic volume | Page section |  | — | Links to instructions for shrinking a basic volume. | — |
| WB.12.06.o21 | Change a drive letter | Page section |  | — | Links to instructions for changing a drive letter. | — |
| WB.12.06.o22 | Disk Management | Built-in Windows tool |  | — | Manage disks and volumes; initialize drives, create and format volumes, change drive letters, and extend or shrink volumes. | — |
| WB.12.06.o23 | Size of the volume | Wizard input | Enter a size in megabytes (MB), or use the maximum size. | The maximum default size is offered. | Sets the size of the volume to create. | — |
| WB.12.06.o24 | Set up a new drive on your computer by initializing a new disk | Disk Management task |  | — | Initializes a new disk to set up a new drive. | — |
| WB.12.06.o25 | Extend a basic volume into unclaimed space in a volume on the same drive | Disk Management task |  | — | Extends a basic volume into unclaimed space on the same drive. | The unclaimed space must be in a volume on the same drive. |
| WB.12.06.o26 | Shrink a basic volume or partition | Disk Management task |  | — | Shrinks a basic volume or partition; the page gives enabling extension into a neighboring partition as an example. | — |
| WB.12.06.o27 | Change a drive letter or assign a new drive letter | Disk Management task |  | — | Changes or assigns a drive letter. | — |
| WB.12.06.o28 | Free up drive space in Windows | Task requiring another Windows tool |  | — | Listed as a common disk-management task completed by using another tool in Windows. | Not completed with Disk Management alone. |
| WB.12.06.o29 | Defragment or optimize your data drives in Windows | Task requiring another Windows tool |  | — | Listed as a common disk-management task completed by using another tool in Windows. | Not completed with Disk Management alone. |
| WB.12.06.o30 | Pool multiple hard drives like a redundant array of independent disks (RAID) with Storage Spaces in Windows | Task requiring another Windows tool |  | — | Listed as a common disk-management task completed by using another tool in Windows. | Uses Storage Spaces in Windows. |

- Confirmations / errors: A typical main drive has Local Disk (C:), EFI System, and Recovery partitions. Local Disk (C:) stores Windows and commonly other apps and files; EFI System supports boot; Recovery stores Windows recovery tools.; Disk Management might show EFI System and Recovery partitions as 100 percent free, but they contain critical files and are usually close to full. The recommended practice is not to modify them.; Disk Management shows drive and partition information, including partition details, statistics, and allocated or used space.; Formatting a volume destroys any data on the partition; back up data you want to save before formatting.; Formatting a volume will destroy any data on the partition; back up any data you want to save before beginning.; If a task reports an error or does not work as expected, the page recommends reviewing Troubleshooting Disk Management, searching Microsoft Support Community, asking a question to Microsoft or the community, or contacting Microsoft Support.; If initialization fails, the page refers readers to Troubleshooting disk management.; If initializing fails, the page directs readers to Troubleshooting disk management.; On a basic disk, the first three new partitions are formatted as primary partitions; beginning with the fourth, each is configured as a logical drive within an extended partition.; Some tasks require system information or resources such as a BitLocker recovery key or a restore point. The page links to articles about creating a recovery drive, creating a restore point, backing up and restoring a PC with Windows Backup, finding a BitLocker recovery key, and finding lost files after upgrading to Windows.; The New Simple Volume Wizard asks you to review your choices before selecting Finish.; The article sections are: “View drives and partitions,” “Prepare resources for tasks,” “Use other tools for tasks,” “Troubleshoot issues,” and “Related content.”; You cannot format a disk or partition that is currently in use, including the partition that contains Windows.
- Keyboard shortcuts: Windows + R opens the Run dialog for entering diskmgmt.msc.; Windows + R opens the Run dialog; type diskmgmt.msc, then select OK.
- SKU / hardware / version conditions: Applies to Windows 11 and Windows 10.; Applies to Windows Server 2025, Windows Server 2022, Windows Server 2019, Windows Server 2016, Windows 11, and Windows 10.; Creating a partition or volume requires administrator sign-in and unallocated disk space or free space within an extended partition.; GPT is recommended for a modern system with UEFI firmware and is described as supporting large drives and more than four partitions.; GPT is suggested for modern systems with UEFI firmware and support for large drives and more than four partitions.; MBR is for older hardware or operating systems that do not support UEFI.; MBR is suggested for older hardware or operating systems that do not support UEFI.; Some USB disks cannot be initialized; these disks are only formatted and then assigned a drive letter.; Some USB disks cannot be initialized; they are only formatted and assigned a drive letter.; The page says Disk Management looks and works the same in Windows 10 and Windows 11, with no feature differences.; When creating partitions on a basic disk, the first three are primary partitions; beginning with the fourth, each is configured as a logical drive within an extended partition.
- Windows 10 differences (legacy): Disk Management looks and works the same in Windows 10 and Windows 11; the page states there are no feature differences between versions.; The page applies to Windows 11 and Windows 10.; The page says there are no feature differences between Windows 10 and Windows 11 for Disk Management.; Windows 10 is listed as an applicable operating system; the page states no Windows 10-specific differences.


### WB.12.07 — System Properties (legacy)

Status: PARTIAL. Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven.

Navigation paths (as stated in sources): `Start > type “Create a restore point” in the search box > select “Create a restore point” from the results`; `Press Win+R > enter systempropertiesprotection.exe > press Enter`; `Select **Start**, search for **Create a restore point**, then select **Create a restore point** from the results to open the **System Properties** dialog box.`; `Use the keyboard shortcut **R**, type `systempropertiesprotection.exe`, and press **Enter**.`

Sources: [Official support article (T1)](https://support.microsoft.com/pl-PL/Windows/Experience/backup-recovery/system-protection); [Official support article (T1)](https://support.microsoft.com/en-us/windows/experience/backup-recovery/system-protection)


| # | Control label (verbatim from source) | Control type | Choices / values | Default (only if stated) | Behaviour / state change | Condition / dependency |
|---|---|---|---|---|---|---|
| WB.12.07.o01 | System Properties | Dialog box |  | — | Contains the System Protection tab. | — |
| WB.12.07.o02 | System Protection | Tab |  | — | Provides controls for configuring System Protection and managing restore points. | — |
| WB.12.07.o03 | Configure... | Button |  | — | Opens configuration for System Protection. | Select it on the System Protection tab. |
| WB.12.07.o04 | Turn on system protection | Option |  | System Protection is not enabled by default. | Enables System Protection. | Available in the configuration dialog. |
| WB.12.07.o05 | Maximum disk space | Slider (no exact label stated) |  | — | Specifies the maximum amount of disk space System Protection can use. | Set after selecting “Turn on system protection”; select “Apply” to apply the configuration. |
| WB.12.07.o06 | Apply | Button |  | — | Applies the configuration. | — |
| WB.12.07.o07 | Create... | Button |  | — | Starts manual restore-point creation. | Select it on the System Protection tab. |
| WB.12.07.o08 | Description | Text entry (no exact label stated) |  | — | Specify a description for the restore point, then select “Create”. | Shown during manual restore-point creation. |
| WB.12.07.o09 | Create | Button |  | — | Creates the restore point. | After specifying a description. |
| WB.12.07.o10 | System Restore | Recovery feature (no control label or navigation path stated) |  | — | Used to revert Windows to a restore point. | — |

- Keyboard shortcuts: The page says: “Use the keyboard shortcut R,” then type `systempropertiesprotection.exe` and press **Enter**.; Win+R opens the Run dialog; enter systempropertiesprotection.exe and press Enter.
- SKU / hardware / version conditions: The page applies to Windows 11 and Windows 10.
- Windows 10 differences (legacy): The page states that it applies to Windows 11 and Windows 10; it gives no differences between them.


---

## 11.14 Completeness Audit — Every WB Item

| ID | Page | Status | Reason |
|---|---|---|---|
| WB | Research B — Windows Settings (26H2) full inventory | PENDING |  |
| WB.01 | System | PARTIAL | Category researched in Part 2; see child items. |
| WB.01.01 | Display | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.01.01 | System > Display > HDR (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.01.02 | System > Display > Night light (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.01.03 | System > Display > Graphics (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.01.04 | System > Display > Scale (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.02 | Sound | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.02.01 | System > Sound > Volume mixer (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.03 | Notifications | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.03.01 | System > Notifications > Do not disturb (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.04 | Focus | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.05 | Power & battery | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.05.01 | System > Power & battery > Energy saver (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.05.02 | System > Power & battery > Screen, sleep & hibernate timeouts (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.06 | Storage | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.06.01 | System > Storage > Cleanup recommendations (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.07 | Nearby sharing | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.08 | Multitasking | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.08.01 | System > Multitasking > Desktops (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.09 | For developers / Advanced | COMPLETE | Official Advanced settings page enumerates the full page contents (updated 2026-02-26); live-build check pending (VP-01). |
| WB.01.10 | Activation | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.11 | Troubleshoot | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.12 | Recovery | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.12.01 | System > Recovery > Fix problems using Windows Update (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.12.02 | System > Recovery > Point-in-time restore (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.13 | Projecting to this PC | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.14 | Remote Desktop | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.15 | Clipboard | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.16 | Optional features (location) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.17 | About | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.18 | Settings Agent (AI) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.01.19 | System > System components (added in Part 2) | PARTIAL | Only independent or preview-era evidence; no official Microsoft documentation found for this page. |
| WB.02 | Bluetooth & devices | PARTIAL | Category researched in Part 2; see child items. |
| WB.02.01 | Devices | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.02.02 | Printers & scanners | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.02.03 | Phone Link / mobile devices | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.02.04 | Cameras | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.02.05 | Mouse | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.02.06 | Touchpad | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.02.07 | Pen & Windows Ink | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.02.08 | Keyboard / Text input | NOT FOUND | No 'Keyboard' page under Bluetooth & devices was documented in the sources inspected; keyboard options are covered under Time & language > Typing (WB.07.03) and Accessibility > Keyboard (WB.09.12). Needs a live-build check. |
| WB.02.09 | AutoPlay | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.02.10 | USB | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.02.11 | Shared Audio | PARTIAL | Only independent or preview-era evidence; no official Microsoft documentation found for this page. |
| WB.02.12 | Bluetooth & devices > Touch (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.03 | Network & internet | PARTIAL | Category researched in Part 2; see child items. |
| WB.03.01 | Wi-Fi | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.03.02 | Ethernet | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.03.03 | VPN | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.03.04 | Mobile hotspot | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.03.05 | Airplane mode | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.03.06 | Proxy | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.03.07 | Dial-up | PARTIAL | Official source confirms the page exists but documents few or none of its controls. |
| WB.03.08 | Advanced network settings | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.03.09 | Data usage | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.03.10 | Network & internet > DNS over HTTPS (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.03.11 | Network & internet > Cellular (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.04 | Personalization | PARTIAL | Category researched in Part 2; see child items. |
| WB.04.01 | Background | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.04.02 | Colors | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.04.03 | Themes | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.04.04 | Dynamic Lighting | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.04.05 | Lock screen | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.04.06 | Text input | PARTIAL | Official source confirms the page exists but documents few or none of its controls. |
| WB.04.07 | Start | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.04.08 | Taskbar | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.04.09 | Fonts | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.04.10 | Device usage | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.05 | Apps | PARTIAL | Category researched in Part 2; see child items. |
| WB.05.01 | Installed apps | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.05.02 | Advanced app settings | PARTIAL | Only community Q&A evidence; official documentation not found. |
| WB.05.03 | Default apps | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.05.04 | Offline maps | PARTIAL | Official source confirms the page exists but documents few or none of its controls. |
| WB.05.05 | Optional features | NOT APPLICABLE | Optional features sits under System > Optional features in current documentation (see WB.01.16). |
| WB.05.06 | Apps for websites | PARTIAL | Official source confirms the page exists but documents few or none of its controls. |
| WB.05.07 | Video playback | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.05.08 | Startup | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.05.09 | Actions | PARTIAL | Official source confirms the page exists but documents few or none of its controls. |
| WB.05.10 | Apps > App reset/repair (Advanced options) (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.06 | Accounts | PARTIAL | Category researched in Part 2; see child items. |
| WB.06.01 | Your info | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.06.02 | Email & accounts | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.06.03 | Sign-in options | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.06.03.01 | Accounts > Sign-in options > Dynamic lock (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.06.04 | Passkeys | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.06.05 | Family | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.06.06 | Windows backup | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.06.07 | Other users | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.06.08 | Access work or school | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.07 | Time & language | PARTIAL | Category researched in Part 2; see child items. |
| WB.07.01 | Date & time | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.07.02 | Language & region | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.07.03 | Typing | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.07.04 | Speech | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.08 | Gaming | PARTIAL | Category researched in Part 2; see child items. |
| WB.08.01 | Game Bar | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.08.02 | Captures | PARTIAL | Only community Q&A evidence; official documentation not found. |
| WB.08.03 | Game Mode | PARTIAL | Official source confirms the page exists but documents few or none of its controls. |
| WB.09 | Accessibility | PARTIAL | Category researched in Part 2; see child items. |
| WB.09.01 | Text size | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.09.02 | Visual effects | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.09.03 | Mouse pointer & touch | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.09.04 | Text cursor | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.09.05 | Magnifier | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.09.06 | Color filters | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.09.07 | Contrast themes | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.09.08 | Narrator | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.09.09 | Audio | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.09.10 | Captions | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.09.11 | Speech / Voice access | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.09.12 | Keyboard | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.09.13 | Mouse | PARTIAL | Official source confirms the page exists but documents few or none of its controls. |
| WB.09.14 | Eye control | PARTIAL | Official source confirms the page exists but documents few or none of its controls. |
| WB.10 | Privacy & security | PARTIAL | Category researched in Part 2; see child items. |
| WB.10.01 | Windows Security | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.10.02 | Find my device | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.10.03 | Device encryption / BitLocker | PARTIAL | Official source confirms the page exists but documents few or none of its controls. |
| WB.10.04 | For developers | NOT APPLICABLE | For developers moved to System > Advanced in 25H2 and later (see WB.01.09). |
| WB.10.05 | Windows permissions (general, speech, inking, diagnostics, activity, search) | PARTIAL | Children WB.10.05.01–.05 documented. |
| WB.10.05.01 | Privacy & security > General (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.10.05.02 | Privacy & security > Speech / Inking & typing personalization (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.10.05.03 | Privacy & security > Diagnostics & feedback (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.10.05.04 | Privacy & security > Activity history (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.10.05.05 | Privacy & security > Search permissions / Searching Windows (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.10.06 | App permissions (location, camera, mic, etc.) | PARTIAL | Children WB.10.06.01–.03 documented. |
| WB.10.06.01 | Privacy & security > Location (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.10.06.02 | Privacy & security > Camera / Microphone (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.10.06.03 | Privacy & security > App permissions (general) (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.10.07 | Privacy & security > Smart App Control (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.11 | Windows Update | PARTIAL | Category researched in Part 2; see child items. |
| WB.11.01 | Updates & pause | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.11.01.01 | Windows Update > Pause updates (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.11.02 | Update history | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.11.03 | Advanced options | PARTIAL | Official source confirms the page exists but documents few or none of its controls. |
| WB.11.04 | Windows Insider Program | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.11.05 | Restart workflows & active hours | PARTIAL | Restart scheduling and active hours are covered in WB.11.01 and WB.11.03. |
| WB.11.06 | Windows Update > Delivery Optimization (added in Part 2) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.12 | Beyond Settings | PARTIAL | Category researched in Part 2; see child items. |
| WB.12.01 | ms-settings URI register (229 distinct URIs on MS Learn) | COMPLETE | Full MS Learn URI table parsed (Appendix 2B). Live resolution test pending (VP-02). |
| WB.12.02 | Control Panel applets | PARTIAL | Official source confirms the page exists but documents few or none of its controls. |
| WB.12.03 | Windows Security app | PARTIAL | See WB.10.01 (Windows Security app). |
| WB.12.04 | Device Manager | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.12.05 | Task Manager (incl. NPU) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.12.06 | Disk Management | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.12.07 | Legacy dialogs (sysdm.cpl etc.) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a live 26300.9711 build, and complete coverage of controls isn't proven. |
| WB.12.08 | Group Policy / MDM-only settings | PENDING | Group Policy / MDM Policy CSP mapping is scheduled for Part 12 (Research I). |
| WB.12.09 | SKU differences (Home/Pro/Enterprise/Education) | PARTIAL | SKU/hardware conditions captured per page where sources state them; no consolidated Home/Pro/Enterprise matrix yet. |

WB totals (leaf and sub-leaf items): PARTIAL 129, COMPLETE 2, NOT FOUND 1, NOT APPLICABLE 2, PENDING 1

---

## 11.15 Contradictions, Gaps, and Uncertain Evidence (Batch B1)

| Gap ID | Type | Description | Evidence | Resolution plan |
|---|---|---|---|---|
| GAP-B1-01 | Contradiction | The taskbar support article says the taskbar sits at the bottom and offers no option to move it, but the 26H2 IT-pro documentation lists top/left/right positions and a smaller taskbar | [Microsoft Support — taskbar](https://support.microsoft.com/help/29280) vs [What's new in 26H2](https://learn.microsoft.com/en-us/windows/whats-new/whats-new-windows-11-version-26h2) | Treat the 26H2 doc as authoritative for the build; the support article is stale. Verify the exact label on a live build (VP-01) |
| GAP-B1-02 | Label uncertainty | The Insider blog doesn't give an exact UI label for "Taskbar position"; Windows Central reports "Choose where the taskbar appears" and "Choose the taskbar size" | [Windows Insider blog](https://blogs.windows.com/windows-insider/2026/05/15/improving-windows-quality-making-taskbar-and-start-more-personal/), [Windows Central](https://www.windowscentral.com/microsoft/windows-11/windows-11-is-dangerously-close-to-letting-you-move-the-taskbar) | Live-build check |
| GAP-B1-03 | Relocation | "For developers" is now System > Advanced (25H2+). The Part 1 checklist listed it under Privacy & security; the item is reclassified as NOT APPLICABLE | [Advanced Windows Settings](https://learn.microsoft.com/en-us/windows/advanced-settings/) | Done |
| GAP-B1-04 | Relocation | Optional features is under System, not Apps | [Microsoft Learn — add/remove features](https://learn.microsoft.com/en-us/windows/client-management/client-tools/add-remove-hide-features?pivots=windows-11) | Done |
| GAP-B1-05 | Deprecated URIs | The URI table marks `ms-settings:backup` (Backup page) and `ms-settings:defaultbrowsersettings` as deprecated in Windows 11, and still lists Cortana, Mixed reality and Surface Hub pages | [Launch the Windows Settings app](https://learn.microsoft.com/en-us/windows/apps/develop/launch/launch-settings-app) | VP-02 resolves each URI on 26300.9711 |
| GAP-B1-06 | Thin evidence | Only community Q&A evidence for Airplane mode details, Dial-up, Advanced app settings, Typing, Game Bar and Captures | See per-page sources | Find official Microsoft support/Learn pages; otherwise live-build check |
| GAP-B1-07 | Missing official doc | System components page: only Canary/23H2-era independent evidence (Winaero, Windows Latest); no official Microsoft page found | Section WB.01.19 | Live-build check |
| GAP-B1-08 | Missing official doc | Apps > Actions page: only independent evidence | Section WB.05.09 | Live-build check |
| GAP-B1-09 | Missing | A separate Bluetooth & devices > Keyboard page wasn't documented in the sources inspected | Section 11.14 (WB.02.08) | Live-build check |
| GAP-B1-10 | Persistence opacity | Several taskbar settings are stored as opaque binary blobs (StuckRects3), and some settings are readable only through the Cloud Data Store reader | [Reference for Windows 11 settings](https://learn.microsoft.com/en-us/windows/apps/develop/settings/settings-windows-11) | Documented as an implementation detail, not a public API; LitricOS design implication in Part 12 |
| GAP-B1-11 | Inaccessible | The `settings-windows` reference URL returned an HTTP client error | Register | Use the settings-windows-11 and settings-common pages instead (done) |
| GAP-B1-12 | Locale sources | Some pages were sourced from non-en-US locale copies (L-03) | Per-page | Re-fetch en-US equivalents |
| GAP-B1-13 | Policy layer | Group Policy and MDM (Policy CSP) controls that lock or hide Settings pages are not yet mapped | — | Part 12 (Research I) |
| GAP-B1-14 | SKU matrix | No consolidated Home/Pro/Enterprise/Education × Copilot+ × region matrix | — | Part 12 + VP-07 |

---

## 11.16 Source Register Status (after Part 2)

| Metric | Count |
|---|---|
| Unique normalized URLs | 718 |
| Unique registrable domains | 49 |
| Pages actually inspected | 315 |
| Domains with at least one inspected page | 26 |
| Search-snippet-only URLs | 402 |
| Inaccessible | 1 |
| Primary sources among inspected | 272 |
| Independent sources among inspected | 43 |
| Progress toward 400-domain target | 49 / 400 — target not met |

The domain count grew slowly in this batch because Windows Settings documentation is concentrated on microsoft.com. That is expected for a primary-source batch. Domain diversity is planned to come from the independent, OEM, accessibility and Linux batches (Part 1, Section 2.3).

---

## Appendix 2A — Settings-to-Persistence Map (registry and data-store), from Microsoft's settings references

These locations are documented by Microsoft for backup/restore data portability ([common reference](https://learn.microsoft.com/en-us/windows/apps/develop/settings/settings-common); [Windows 11 reference](https://learn.microsoft.com/en-us/windows/apps/develop/settings/settings-windows-11)). They describe where a setting's state is stored. They are not a supported write API: changing the values directly may not refresh the shell UI.


### Source: [settings-common](https://learn.microsoft.com/en-us/windows/apps/develop/settings/settings-common)

| Section | Store (key path / type) | Value | Type | Data | Description |
|---|---|---|---|---|---|
| AppList Backup Data | `Windows.Data.Apps.AppMetaData` | `appId` | wstring |  | PackageFamilyName (packaged) or ProductCode (unpackaged). |
| AppList Backup Data | `Windows.Data.Apps.AppMetaData` | `installSource` | wstring |  | Description of type of installer for the app. See *installSource* for supported values. |
| AppList Backup Data | `Windows.Data.Apps.AppMetaData` | `lightIconInfo` | IconInfo |  | Information about the light icon. |
| AppList Backup Data | `Windows.Data.Apps.AppMetaData` | `darkIconInfo` | IconInfo |  | Information about the dark icon. |
| AppList Backup Data | `Windows.Data.Apps.AppMetaData` | `appName` | wstring |  | App display name. |
| AppList Backup Data | `Windows.Data.Apps.AppMetaData` | `publisher` | wstring |  | Publishers name of the app from Add Remove Programs. |
| AppList Backup Data | `Windows.Data.Apps.AppMetaData` | `lastLaunchTime` | uint64 |  | This is a calculated value that represents the last time the app was launched. |
| AppList Backup Data | `Windows.Data.Apps.AppMetaData` | `appVersion` | wstring |  | Version of the app from Add Remove Programs. |
| AppList Backup Data | `Windows.Data.Apps.AppMetaData` | `appLanguage` | wstring |  | Language list in Add Remove Programs. |
| AppList Backup Data | `Windows.Data.Apps.AppMetaData` | `appArch` | wstring |  | Architecture specified in Add Remove Programs. |
| AppList Backup Data | `Windows.Data.Apps.AppMetaData` | `reinstallId` | wstring |  | Reinstall ID specified in Add Remove Programs. |
| AppList Backup Data | `Windows.Data.Apps.AppMetaData` | `productUrl` | wstring |  | Product URL Specified in Add Remove Programs. |
| AppList Backup Data | `Windows.Data.Apps.AppMetaData` | `isPinned` | bool |  | Boolean indicating if this app was pinned to the start menu. |
| AppList Backup Data | `Windows.Data.Apps.AppMetaData` | `wingetID` | wstring |  | Identifier to indicate if this app can be installed through winget, and the winget ID. |
| AppList Backup Data | `Windows.Data.Apps.AppMetaData` | `wingetSource` | wstring |  | Specifies where the app was sourced from through the Winget APIs. See *wingetSource* for supported values. |
| AppList Backup Data | `Windows.Data.Apps.IconInfo` | `appIconAssetId` | wstring |  | The ID representing an icon from the cloud. The appIconAssetId represents the icon stored in the cloud which was generated from the app installer either through add remove programs metadata or AppxManifest files. |
| AppList Backup Data | `Windows.Data.Apps.IconInfo` | `isPlated` | bool |  | Plated or not. |
| AppList Backup Data | `Windows.Data.Apps.ShortcutInfo` | `targetPath` | wstring |  | Link to the executable that launching the tile will shell execute. |
| AppList Backup Data | `Windows.Data.Apps.ShortcutInfo` | `shortcutArgs` | wstring |  | Arguments provided on launch. |
| AppList Backup Data | `Windows.Data.Apps.AppLevelTileInfo` | `tileId` | wstring |  | UnifiedTileIdentifier for tile |
| AppList Backup Data | `Windows.Data.Apps.AppLevelTileInfo` | `displayName` | wstring |  | String displayed on tile |
| AppList Backup Data | `Windows.Data.Apps.AppLevelTileInfo` | `sortName` | wstring |  | String value used for sorting in search |
| AppList Backup Data | `Windows.Data.Apps.AppLevelTileInfo` | `packageId` | wstring |  | PackageFamilyName (packaged), ProductCode (unpackaged), empty for unmapped products (appId is specified in **AppMetaData**) |
| AppList Backup Data | `Windows.Data.Apps.AppLevelTileInfo` | `shortcut` | ShortcutInfo |  | Shortcut information. |
| AppList Backup Data | `Windows.Data.Apps.AppLevelTileInfo` | `suiteName` | wstring |  | String name for a collection of apps. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `name` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `path` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `osComponent` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `size` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `magic` | wstring |  | Read from App compatibility JSON. For information on the PE header, see PE Format. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `peHeaderHash` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `sizeOfImage` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `peChecksum` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `linkDate` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `linkerVersion` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `binFileVersion` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `binProductVersion` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `binaryType` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `created` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `modified` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `lastAccessed` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `verLanguage` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `id` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `switchBackContext` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `sigDisplayName` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `sigPublisherName` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `sigMoreInfoURL` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `fileVersion` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `companyName` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `fileDescription` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `internalName` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `legalCopyright` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `originalFileName` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `productName` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `productVersion` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `peImageType` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `peSubsystem` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `runLevel` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `uiAccess` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `crcChecksum` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `clrVersion` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `boeProgramId` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `boeProgramName` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `boeProgramPublisher` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `boeProgramVersion` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `boeProgramLanguage` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `fileSize` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `peCharacteristics` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `sha256` | wstring |  | Read from App compatibility JSON. Optional. |
| AppList Backup Data | `Windows.Data.Apps.FileInfo` | `aumid` | wstring |  | Application user model ID. Optional. For more information, see Find the Application User Model ID of an installed app |
| AppList Backup Data | `Windows.Data.Apps.AppCompatInfo` | `programId` | wstring |  | Read from App compatibility JSON. |
| AppList Backup Data | `Windows.Data.Apps.AppCompatInfo` | `compatFlags` | wstring |  | Read from App compatibility JSON. |
| AppList Backup Data | `Windows.Data.Apps.AppCompatInfo` | `restoreActions` | wstring |  | Read from App compatibility JSON |
| AppList Backup Data | `Windows.Data.Apps.AppCompatInfo` | `files` | vector <FileInfo> |  | Read from App compatibility JSON. |
| AppList Backup Data | `Windows.Data.Apps.DeviceMetadata` | `userIntent` | uint32 |  | This is a bit array of values specifying user intent during Windows setup. See *userIntent* values. This data is read from HKCU\Software\Microsoft\Windows\CurrentVersion\CloudExperienceHost\Intent |
| AppList Backup Data | `Windows.Data.Apps.DeviceMetadata` | `predictedUserIntent` | uint32 |  | Windows sets flag to indicate that the user had one or more apps that are a signal of a developer. |
| AppList Backup Data | `Windows.Data.Apps.DeviceMetadata` | `devModeEnabled` | bool |  | Whether the user has specified Developer Mode from Windows Settings. This data is read from from HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock |
| Autoplay | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\AutoplayHandlers` | `DisableAutoplay` | REG_DWORD | 0/1 | Toggle the use of autoplay for all media and devices. |
| Autoplay | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\AutoplayHandlers` | `UserChosenExecuteHandlers\CameraAlternate\ShowPicturesOnArrival` | REG_SZ | A string. | User selected default autoplay behavior for removable drive. See table below for supported values. |
| Autoplay | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\AutoplayHandlers` | `UserChosenExecuteHandlers\ StorageOnArrival` | REG_SZ | A string. | User selected default autoplay behavior for memory card. See table below for supported values. |
| Autoplay | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\AutoplayHandlers` | `EventHandlersDefaultSelection\CameraAlternate\ShowPicturesOnArrival` | REG_SZ | A string. | System default autoplay behavior for removable drive. See table below for supported values. |
| Autoplay | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\AutoplayHandlers` | `EventHandlersDefaultSelection\StorageOnArrival` | REG_SZ | A string. | System default autoplay behavior for memory card. See table below for supported values. |
| Background | `Windows.Data.Background.DesktopWallpaper` | `kind` | WallpaperKind |  | Specifies whether current background is set as Wallpaper or Solid Color or Slideshow or Spotlight. |
| Background | `Windows.Data.Background.DesktopWallpaper` | `position` | WallpaperPosition |  | Specifies how wallpaper or slideshow images are positioned on background. |
| Background | `Windows.Data.Background.DesktopWallpaper` | `color` | Windows.Data.Common.Color |  | Specifies the solid color value if background is selected as Solid Color |
| Background | `Windows.Data.Background.DesktopWallpaper` | `itemId` | wstring |  | The unique ID for the wallpaper or slideshow uploaded during backup. |
| Background | `Windows.Data.Background.DesktopWallpaper` | `contentUri` | wstring |  | The url for the wallpaper or slideshow uploaded during backup. |
| Background | `Windows.Data.Background.DesktopWallpaper` | `intervalInSeconds` | uint64 |  | The interval between images of slideshow if background is selected as slideshow. |
| Background | `Windows.Data.Background.DesktopWallpaper` | `shuffle` | bool |  | Signifies whether slideshow images are shuffled if background is selected as slideshow |
| Background | `Windows.Data.Background.DesktopWallpaper` | `syncRootRelativePath` | wstring |  | Signifies path to slideshow folder if background is selected as slideshow. |
| Calling | `Windows.data.calling.Settings` | `perKeySettings` | map<wstring, KeyPathSettings> |  | A map of per-key settings |
| Calling | `Windows.data.calling.KeyPathSettings` | `values` | map<wstring, wstring> |  | A map of key/path settings |
| Calling | `Windows.data.calling.callhistoryItem` | `uniqueId` | String |  | Unique identifier for the record. |
| Calling | `Windows.data.calling.callhistoryItem` | `phoneNumber` | String |  | Phone number of the caller. |
| Calling | `Windows.data.calling.callhistoryItem` | `calltype` | Enum CallType |  | Possible values: Outgoing, IncomingAnswered, IncomingMissed, IncomingRinging, OutgoingMissed, OutgoingRinging. |
| Calling | `Windows.data.calling.callhistoryItem` | `voicemailCall` | Bool |  | Indicates whether the call was a voice mail. |
| Calling | `Windows.data.calling.callhistoryItem` | `videocall` | Bool |  | Indicates whether the call was a video call. |
| Calling | `Windows.data.calling.callhistoryItem` | `Seen` | Bool |  | Indicates whether the call history was seen by the user. |
| Calling | `Windows.data.calling.callhistoryItem` | `callerIdBlocked` | Bool |  | Indicates whether the call was blocked. |
| Calling | `Windows.data.calling.callhistoryItem` | `emergencycall` | Bool |  | Indicates whether call was an emergency call. |
| Calling | `Windows.data.calling.callhistoryItem` | `linenumber` | String |  | The number of the phone line that received the call. |
| Calling | `Windows.data.calling.callhistoryItem` | `lineName` | String |  | The phone line's name. |
| Calling | `Windows.data.calling.callhistoryItem` | `callerLocation` | String |  | Caller location. |
| Calling | `Windows.data.calling.callhistoryItem` | `callerCategory` | String |  | Caller category. |
| Calling | `Windows.data.calling.callhistoryItem` | `callerCategoryDescription` | String |  | Caller category description. |
| Calling | `Windows.data.calling.callhistoryItem` | `Calltimestamp` | Unit64 |  | unix time stamp. |
| Calling | `Windows.data.calling.callhistoryItem` | `Callarrivaltime` | Unit64 |  | Unix time the call arrived on the device. |
| Calling | `Windows.data.calling.callhistoryItem` | `callEndTime` | Unit64 |  | Unix time stamp when the call was ended. |
| Calling | `Windows.Data.calling.callhistory` | `historyItems` | Map<string, CallHistoryItem> |  | A collection of call history items where the keys are each history item's UniqueId. |
| Calling | `Windows.Data.calling.callhistory` | `highestSequenceNumber` | Unit32 |  | Highest sequence number issued, used for internal business logic. |
| Calling | `Windows.Data.calling.callfavorites` | `favoriteItems` | vector<CallFavoriteItem> |  | A collection of calling favorites. |
| Calling | `Windows.data.calling.CallFavoriteItem` | `displayName` | String |  | Display name of the favorite. |
| Calling | `Windows.data.calling.CallFavoriteItem` | `lineNumber` | String |  | Line number the favorite is associated with. |
| Calling | `Windows.data.calling.CallFavoriteItem` | `phoneNumberName` | Uint32 |  | Phone number name. |
| Calling | `Windows.data.calling.CallFavoriteItem` | `remoteIdHash` | String |  | Remote ID hash. |
| Calling | `Windows.data.calling.CallFavoriteItem` | `propHash` | String |  | Property hash. |
| Calling | `Windows.data.calling.CallFavoriteItem` | `isVideoCall` | Bool |  | Indicates whether the item is a video call. |
| Calling | `Windows.data.calling.CallFavoriteItem` | `isPublicSwitchTelephoneNetwork` | Bool |  | Indicates whether the item is a telephone call. |
| Calling | `Windows.data.calling.CallFavoriteItem` | `applicationId` | String |  | Application ID. |
| Calling | `Windows.data.calling.CallFavoriteItem` | `callbackToken` | String |  | Callback token. |
| Calling | `Windows.data.calling.CallFavoriteItem` | `UniqueId` | Unit64 |  | A unique identifier for the item. |
| Contrast themes | `Windows.Data.ContrastThemes.SynchedTheme` | `type` | CurrentThemeType |  | Specifies whether current theme is personalized theme or inbox high contrast theme or customized high contrast theme. |
| Contrast themes | `Windows.Data.ContrastThemes.SynchedTheme` | `currentThemePath` | wstring |  | The complete path to the currently applied theme file. |
| Contrast themes | `Windows.Data.ContrastThemes.SynchedTheme` | `contrastThemePath` | wstring |  | The complete path to the last applied high contrast theme file.This can be same as of *currentThemePath* if current theme is high contrast. |
| Contrast themes | `Windows.Data.ContrastThemes.SynchedTheme` | `baseContrastThemeName` | wstring |  | Specifies the base in-box high contrast theme is applied. Any customization may have been applied on top of these themes. Supported values are "High Contrast #1", " or "High Contrast #2", "High Contrast Black", or "High Contrast White" |
| Contrast themes | `Windows.Data.ContrastThemes.SynchedTheme` | `customThemeName` | wstring |  | The user-defined name for a customized contrast theme. |
| Contrast themes | `Windows.Data.ContrastThemes.SynchedTheme` | `rgbBackground` | Windows.Data.Common.Color |  | Theme background color. |
| Contrast themes | `Windows.Data.ContrastThemes.SynchedTheme` | `rgbText` | Windows.Data.Common.Color |  | Theme text color. |
| Contrast themes | `Windows.Data.ContrastThemes.SynchedTheme` | `rgbHyperlink` | Windows.Data.Common.Color |  | Theme hyperlink color. |
| Contrast themes | `Windows.Data.ContrastThemes.SynchedTheme` | `rgbInactiveText` | Windows.Data.Common.Color |  | Theme inactive text color. |
| Contrast themes | `Windows.Data.ContrastThemes.SynchedTheme` | `rgbSelectedText1` | Windows.Data.Common.Color |  | Signifies color value of Text of highlighted text. |
| Contrast themes | `Windows.Data.ContrastThemes.SynchedTheme` | `rgbSelectedText2` | Windows.Data.Common.Color |  | Signifies color value of Text of highlight. |
| Contrast themes | `Windows.Data.ContrastThemes.SynchedTheme` | `rgbButtonText1` | Windows.Data.Common.Color |  | Signifies color value of Text of button text. |
| Contrast themes | `Windows.Data.ContrastThemes.SynchedTheme` | `rgbButtonText2` | Windows.Data.Common.Color |  | Signifies color value of Face of button. |
| Contrast themes | `Windows.Data.Common.Color` | `red` | uint8 |  | Red channel value of an RGBA color. |
| Contrast themes | `Windows.Data.Common.Color` | `green` | uint8 |  | Green channel value of an RGBA color. |
| Contrast themes | `Windows.Data.Common.Color` | `blue` | uint8 |  | Blue channel value of an RGBA color. |
| Contrast themes | `Windows.Data.Common.Color` | `alpha` | uint8 |  | Alpha channel value of an RGBA color. |
| Date and Time | `HKLM\SYSTEM\CurrentControlSet\Services\tzautoupdate` | `start` | REG_DWORD | 3/4 | 3: Set timezone toggle value on. 4: Set Timezone toggle value off. |
| Date and Time | `HKLM\SYSTEM\CurrentControlSet\Control\TimeZoneInformation` | `timeZone` | REG_SZ | System timezone as string. | The currently set time zone for the system. |
| Do not disturb | `Windows.Data.DoNotDisturb.BoolWithMetadata` | `value` | bool |  | The boolean value. |
| Do not disturb | `Windows.Data.DoNotDisturb.BoolWithMetadata` | `changeReason` | ChangeReason |  | The reason a user changed their profile. |
| Do not disturb | `Windows.Data.DoNotDisturb.QuietHoursProfile` | `isInitialized` | bool |  | CDS data initialize status. |
| Do not disturb | `Windows.Data.DoNotDisturb.QuietHoursProfile` | `settings` | map<uint64, bool> |  | Map of the boolean state of the settings. Specifies when a given settings checkbox is checked or unchecked. |
| Do not disturb | `Windows.Data.DoNotDisturb.QuietHoursProfile` | `allowedContacts` | set<wstring> |  | The set of allowed contacts as a list. |
| Do not disturb | `Windows.Data.DoNotDisturb.QuietHoursProfile` | `allowedApps` | set<wstring> |  | The list of app names to set notifications on. |
| Do not disturb | `Windows.Data.DoNotDisturb.QuietHoursProfile` | `defaultAllowedAppsRemoved` | set<wstring> |  | The list of names of the default app names removed. |
| Do not disturb | `Windows.Data.DoNotDisturb.QuietHoursSettings` | `selectedProfile` | wstring |  | The string value of the user profile. "Unrestricted" or "Priority Only" |
| Do not disturb | `Windows.Data.DoNotDisturb.QuietHoursSettings` | `shouldShowSummaryToast` | BoolWithMetadata |  | Sets whether the summary toast should be shown and the associated reason. |
| Do not disturb | `Windows.Data.DoNotDisturb.QuietMoment` | `isEnabled` | bool |  | Settings enabled or disabled. |
| Do not disturb | `Windows.Data.DoNotDisturb.QuietMoment` | `assignedProfile` | wstring |  | The value of the user profile set by automatic rules. "Unrestricted", "Priority Only", or "Alarms Only" |
| Do not disturb | `Windows.Data.DoNotDisturb.QuietMoment` | `startTime` | TimeSpan |  | When automatic rules should start. |
| Do not disturb | `Windows.Data.DoNotDisturb.QuietMoment` | `endTime` | TimeSpan |  | When automatic rules should end. |
| Do not disturb | `Windows.Data.DoNotDisturb.QuietMoment` | `repeatType` | uint32 |  | How frequently the rules should apply. "Daily", "Weekend", or "Weekdays" |
| Do not disturb | `Windows.Data.DoNotDisturb.QuietMoment` | `shouldShowActiveToast` | BoolWithMetadata |  | Sets whether the active toast should be shown and the associated reason. |
| File Explorer Classic | `Windows.Data.FileExplorerClassic.ShellStateSetting` | `bmigratedFromSSF` | bool |  | Indicates whether the shell state is migrated from legacy setting framework to current solution. |
| File Explorer Classic | `Windows.Data.FileExplorerClassic.ShellStateSetting` | `bshowAllObjects` | bool |  | Indicates whether Shell state should show all objects. |
| File Explorer Classic | `Windows.Data.FileExplorerClassic.ShellStateSetting` | `bshowExtensions` | bool |  | Indicates whether Shell state should show all extensions. |
| File Explorer Classic | `Windows.Data.FileExplorerClassic.ShellStateSetting` | `bshowCompColor` | bool |  | Indicates whether Shell state should show all colors. |
| File Explorer Classic | `Windows.Data.FileExplorerClassic.ShellStateSetting` | `bdoubleClickInWebView` | bool |  | Indicates whether Shell state should show all objects. |
| File Explorer Classic | `Windows.Data.FileExplorerClassic.ShellStateSetting` | `bdontPrettyPath` | bool |  | Indicates whether Shell state should show pretty path |
| File Explorer Classic | `Windows.Data.FileExplorerClassic.ShellStateSetting` | `showInfoTip` | bool |  | Indicates whether Shell state should show info tip. |
| File Explorer Classic | `Windows.Data.FileExplorerClassic.ShellStateSetting` | `noConfirmRecycle` | bool |  | Indicates whether Shell state should should confirm recycle option. |
| File Explorer Classic | `Windows.Data.FileExplorerClassic.ShellStateSetting` | `showSuperHidden` | bool |  | Indicates whether Shell state should show super hidden. |
| File Explorer Classic | `Windows.Data.FileExplorerClassic.ShellStateSetting` | `sepProcess` | bool |  | Indicates whether Shell state should show the separation process. |
| File Explorer Classic | `Windows.Data.FileExplorerClassic.ShellStateSetting` | `iconsOnly` | bool |  | Indicates whether Shell state should show icons only. |
| File Explorer Classic | `Windows.Data.FileExplorerClassic.ShellStateSetting` | `showTypeOverlay` | bool |  | Indicates whether Shell state should show type overlay. |
| File Explorer Classic | `Windows.Data.FileExplorerClassic.CabinetStateSettings` | `migratedFromSSF` | bool |  | Indicates whether the cabinet state is migrated from legacy setting framework to current solution. |
| File Explorer Classic | `Windows.Data.FileExplorerClassic.CabinetStateSettings` | `fullPathTitle` | bool |  | Indicates whether the cabinet state setting should show full path title. |
| File Explorer Classic | `Windows.Data.FileExplorerClassic.CabinetStateSettings` | `saveLocalView` | bool |  | Indicates whether the cabinet state setting should save local view. |
| File Explorer Classic | `Windows.Data.FileExplorerClassic.CabinetStateSettings` | `newWindowMode` | bool |  | Indicates whether the cabinet state setting should open in new window mode. |
| File Explorer Classic | `Windows.Data.FileExplorerClassic.AdvancedSettings` | `alwaysShowMenus` | bool |  | Indicates whether advanced settings should always show menu. |
| File Explorer Classic | `Windows.Data.FileExplorerClassic.AdvancedSettings` | `folderContentsInfoTip` | bool |  | Indicates whether advanced settings should show folder contents info tip. |
| File Explorer Classic | `Windows.Data.FileExplorerClassic.AdvancedSettings` | `hideDrivesWithNoMedia` | bool |  | Indicates whether advanced settings should hide the drives with no media present. |
| File Explorer Classic | `Windows.Data.FileExplorerClassic.AdvancedSettings` | `navPaneExpandToCurrentFolder` | bool |  | Indicates whether advanced settings should expand the navigation pane to current folder. |
| File Explorer Classic | `Windows.Data.FileExplorerClassic.AdvancedSettings` | `navPaneShowAllFolders` | bool |  | Indicates whether advanced settings should expand the navigation panel to show all folders. |
| File Explorer Classic | `Windows.Data.FileExplorerClassic.AdvancedSettings` | `navPaneShowFavorites` | bool |  | Indicates whether advanced settings should expand the navigation panel to show all favorites. |
| File Explorer Classic | `Windows.Data.FileExplorerClassic.AdvancedSettings` | `persistBrowsers` | bool |  | Indicates advanced settings for file explorer should persist browsers. |
| File Explorer Classic | `Windows.Data.FileExplorerClassic.AdvancedSettings` | `sharingWizardOn` | bool |  | Indicates whether sharing wizard is on for advance |
| File Explorer | `Windows.Data.FileExplorerClassic.ShellStateSettings` | `showSuperHidden` | bool |  | Indicates whether Shell state should show super hidden. |
| File Explorer | `Windows.Data.FileExplorerClassic.ShellStateSettings` | `sepProcess` | bool |  | Indicates whether Shell state should show the separation process. |
| File Explorer | `Windows.Data.FileExplorerClassic.ShellStateSettings` | `iconsOnly` | bool |  | Indicates whether Shell state should show icons only. |
| File Explorer | `Windows.Data.FileExplorerClassic.ShellStateSettings` | `showTypeOverlay` | bool |  | Indicates whether Shell state should show type overlay. |
| File Explorer | `Windows.Data.FileExplorerClassic.CabinetStateSettings` | `migratedFromSSF` | bool |  | Indicates whether the cabinet state is migrated from legacy setting framework to current solution. |
| File Explorer | `Windows.Data.FileExplorerClassic.CabinetStateSettings` | `fullPathTitle` | bool |  | Indicates whether the cabinet state setting should show full path title. |
| File Explorer | `Windows.Data.FileExplorerClassic.CabinetStateSettings` | `saveLocalView` | bool |  | Indicates whether the cabinet state setting should save local view. |
| File Explorer | `Windows.Data.FileExplorerClassic.CabinetStateSettings` | `newWindowMode` | bool |  | Indicates whether the cabinet state setting should open in new window mode. |
| File Explorer | `Windows.Data.FileExplorerClassic.AdvancedSettings` | `alwaysShowMenus` | bool |  | Indicates whether advanced settings should always show menu. |
| File Explorer | `Windows.Data.FileExplorerClassic.AdvancedSettings` | `folderContentsInfoTip` | bool |  | Indicates whether advanced settings should show folder contents info tip. |
| File Explorer | `Windows.Data.FileExplorerClassic.AdvancedSettings` | `hideDrivesWithNoMedia` | bool |  | Indicates whether advanced settings should hide the drives with no media present. |
| File Explorer | `Windows.Data.FileExplorerClassic.AdvancedSettings` | `navPaneExpandToCurrentFolder` | bool |  | Indicates whether advanced settings should expand the navigation pane to current folder. |
| File Explorer | `Windows.Data.FileExplorerClassic.AdvancedSettings` | `navPaneShowAllFolders` | bool |  | Indicates whether advanced settings should expand the navigation panel to show all folders. |
| File Explorer | `Windows.Data.FileExplorerClassic.AdvancedSettings` | `navPaneShowFavorites` | bool |  | Indicates whether advanced settings should expand the navigation panel to show all favorites. |
| File Explorer | `Windows.Data.FileExplorerClassic.AdvancedSettings` | `persistBrowsers` | bool |  | Indicates advanced settings for file explorer should persist browsers. |
| File Explorer | `Windows.Data.FileExplorerClassic.AdvancedSettings` | `sharingWizardOn` | bool |  | Indicates whether sharing wizard is on for advanced settings. |
| File Explorer | `Windows.Data.FileExplorerClassic.AdvancedSettings` | `showDriveLetters` | bool |  | Indicates whether drives letters are shown for advanced settings. |
| File Explorer | `Windows.Data.FileExplorerClassic.AdvancedSettings` | `showPreviewHandlers` | bool |  | Indicates whether to show preview handlers. |
| File Explorer | `Windows.Data.FileExplorerClassic.AdvancedSettings` | `typeAhead` | bool |  | Indicates whether to show type ahead. |
| File Explorer | `Windows.Data.FileExplorerClassic.AdvancedSettings` | `showStatusBar` | bool |  | Indicates whether the file explorer should show status bar. |
| File Explorer | `Windows.Data.FileExplorerClassic.AdvancedSettings` | `showLibraries` | bool |  | Indicates whether the file explorer advanced settings should show libraries. |
| File Explorer | `Windows.Data.FileExplorerClassic.AdvancedSettings` | `showCompColor` | bool |  | Indicates whether to show comp color for advanced settings. |
| File Explorer | `Windows.Data.FileExplorerClassic.ExplorerSettings` | `underlineIconsAlways` | bool |  | Indicates whether Explorer settings should always underline icons. |
| File Explorer | `Windows.Data.FileExplorerClassic.ExplorerSettings` | `underlineIconsNever` | bool |  | Indicates whether Explorer settings should never underline icons. |
| File Explorer | `Windows.Data.FileExplorerClassic.ExplorerSettings` | `underlineIconsAsBrowser` | bool |  | Indicates whether Explorer settings should underline icons as browser. |
| File Explorer | `Windows.Data.FileExplorerClassic.SearchSettings` | `searchOnly` | bool |  | Indicates whether search setting show search only. |
| File Explorer | `Windows.Data.FileExplorerClassic.SearchSettings` | `wholeFileSystem` | bool |  | Indicates whether search setting show the whole file system. |
| File Explorer | `Windows.Data.FileExplorerClassic.SearchSettings` | `systemFolders` | bool |  | Indicates whether search setting show system folders. |
| File Explorer | `Windows.Data.FileExplorerClassic.SearchSettings` | `archivedFiles` | bool |  | Indicates whether search setting show archived files. |
| File Explorer | `Windows.Data.FileExplorerClassic.RegistrySettings` | `RibbonQat` | blob |  | A single blob. |
| File Explorer | `Windows.Data.FileExplorerClassic.RegistrySettings` | `detailsPreviewPaneSettings` | unit32 |  | Details about preview pane settings. |
| File Explorer | `Windows.Data.FileExplorerClassic.RegistrySettings` | `readingPaneSettings` | unit32 |  | Indicates reading pane settings. |
| File Explorer | `Windows.Data.FileExplorerClassic.RegistrySettings` | `navigationPaneVisible` | bool |  | Indicates whether navigation pane is visible or not. |
| File Explorer | `Windows.Data.FileExplorerClassic.FolderOptionGeneralSettings` | `defaultOpenLocation` | unit8 | Default is home (0). | Indicates whether file explorer can be opened to either home or this PC. |
| File Explorer | `Windows.Data.FileExplorerClassic.FolderOptionGeneralSettings` | `browseFoldersInNewWindow` | bool |  | Indicates whether to browse folder in new window. |
| File Explorer | `Windows.Data.FileExplorerClassic.FolderOptionGeneralSettings` | `useDoubleClickToOpen` | bool |  | Indicates whether to use double click to open. |
| File Explorer | `Windows.Data.FileExplorerClassic.FolderOptionGeneralSettings` | `showRecentlyUsedFiles` | bool |  | Indicates whether to show recently used files. |
| File Explorer | `Windows.Data.FileExplorerClassic.FolderOptionGeneralSettings` | `showFrequentlyUsedFolders` | bool |  | Indicates whether to show frequently used folders. |
| File Explorer | `Windows.Data.FileExplorerClassic.FolderOptionGeneralSettings` | `showFilesFromOffice` | bool |  | Indicates whether to show files from MS office. |
| File Explorer | `Windows.Data.FileExplorerClassic.FolderOptionsAdvancedSettings` | `alwaysShowIconsNeverThumbnails` | bool |  | This is used to show icons only. |
| File Explorer | `Windows.Data.FileExplorerClassic.FolderOptionsAdvancedSettings` | `compactView` | bool |  | Decrease space between items. |
| File Explorer | `Windows.Data.FileExplorerClassic.FolderOptionsAdvancedSettings` | `displayFileIconOnThumbnails` | bool |  | Indicates whether to display file icon on thumbnails. |
| File Explorer | `Windows.Data.FileExplorerClassic.FolderOptionsAdvancedSettings` | `displayFileSizeInfoInFolderTips` | bool |  | Indicates whether to display file size information in folder tips. |
| File Explorer | `Windows.Data.FileExplorerClassic.FolderOptionsAdvancedSettings` | `displayFullFilePath` | bool |  | Show folder contents info tip. |
| File Explorer | `Windows.Data.FileExplorerClassic.FolderOptionsAdvancedSettings` | `showSysHiddenFiles` | bool |  | Show/hide hidden files and folders. |
| File Explorer | `Windows.Data.FileExplorerClassic.FolderOptionsAdvancedSettings` | `hideEmptyDrives` | bool |  | Hide drives with no media. |
| File Explorer | `Windows.Data.FileExplorerClassic.FolderOptionsAdvancedSettings` | `showExtensions` | bool |  | Show/hide extensions of known file types. |
| File Explorer | `Windows.Data.FileExplorerClassic.FolderOptionsAdvancedSettings` | `hideFolderMergeConflicts` | bool |  | Indicates whether to show/hide folder merge conflicts. |
| File Explorer | `Windows.Data.FileExplorerClassic.FolderOptionsAdvancedSettings` | `showSuperHiddenOSProtectedFiles` | bool |  | Show/hide OS protected files. |
| File Explorer | `Windows.Data.FileExplorerClassic.FolderOptionsAdvancedSettings` | `colorEncryptedCompressedFiles` | bool |  | Show encrypted and compressed files in color. |
| File Explorer | `Windows.Data.FileExplorerClassic.FolderOptionsAdvancedSettings` | `showPopupDescription` | bool |  | Indicates pop up descriptions. |
| File Explorer | `Windows.Data.FileExplorerClassic.FolderOptionsAdvancedSettings` | `showSyncProviderNotification` | bool |  | Indicates notification provider sync notification. |
| File Explorer | `Windows.Data.FileExplorerClassic.FolderOptionsAdvancedSettings` | `useCheckboxesForSelection` | bool |  | Used to show check boxes for selection. |
| File Explorer | `Windows.Data.FileExplorerClassic.FolderOptionsAdvancedSettings` | `useSharingWizard` | bool |  | Sharing wizard on/off. |
| File Explorer | `Windows.Data.FileExplorerClassic.FolderOptionsAdvancedSettings` | `navPaneAlwaysShowAvailablityStatus` | bool |  | Show Navigation pane all cloud states. |
| File Explorer | `Windows.Data.FileExplorerClassic.FolderOptionsAdvancedSettings` | `navPaneShowLibraries` | bool |  | Show navigation pane libraries. |
| File Explorer | `Windows.Data.FileExplorerClassic.FolderOptionsAdvancedSettings` | `navPaneShowNetwork` | bool |  | Show pane navigation network. |
| File Explorer | `Windows.Data.FileExplorerClassic.FolderOptionsAdvancedSettings` | `navPaneShowThisPC` | bool |  | This is used to show navigation pane to show current folder. |
| File Explorer | `Windows.Data.FileExplorerClassic.RecycleBinSettings` | `displayDeleteConfirmationDialog` | bool |  | Indicates whether to show delete confirmation dialog |
| File Explorer | `Windows.Data.FileExplorerClassic.DesktopIconSettings` | `viewAutoArrangeIcons` | bool |  | Indicates values for AutoArrangement of icons. |
| File Explorer | `Windows.Data.FileExplorerClassic.DesktopIconSettings` | `viewAlignIconsToGrid` | bool |  | Indicates values for snapping to grid. |
| File Explorer | `Windows.Data.FileExplorerClassic.DesktopIconSettings` | `viewShowDesktopIcons` | bool |  | Indicates values to view desktop icons. |
| File Explorer | `Windows.Data.FileExplorerClassic.DesktopIconSettings` | `sortColPropertyKeyFmtid` | wstring | L"B725F130-47EF-101A-A5F1-02608C9EEBAC". |  |
| File Explorer | `Windows.Data.FileExplorerClassic.DesktopIconSettings` | `sortColPropertyKeyPid` | Unit32 | 10. |  |
| File Explorer | `Windows.Data.FileExplorerClassic.DesktopIconSettings` | `sortColDirection` | bool | 1=Ascending -1=Descending Default value is Ascending (true). |  |
| Input Method Editors (IME) | `Windows.Data.Input.HistoryFiles` | `InputDataFiles` | Map<string,FilePathInfo> |  | A map of input history FilePathInfo structs where the key represents a relative folder path. |
| Input Method Editors (IME) | `Windows.Data.Input.FilePathInfo` | `filePath` | Map<string,fileData> |  | A map of FileData structs where the key is the file name. |
| Internet Explorer | `Windows.Data.InternetExplorer.Favorites` | `favoriteSettings` | vector<SettingUnit> |  | SettingUnit structures related to Internet Explorer favorites. |
| Internet Explorer | `Windows.Data.InternetExplorer.TypedURLS` | `typedURLSettings` | vector<SettingUnit> |  | SettingUnit structures related to Internet Explorer TypedURLS. |
| Internet Explorer | `Windows.Data.InternetExplorer.BrowserHistory` | `BrowserHistory` | vector<SettingUnit> |  | SettingUnit structures related to Internet Explorer browser history. |
| Internet Explorer | `Windows.Data.InternetExplorer.AutoComplete` | `AutoCompleteSetting` | vector<SettingUnit> |  | SettingUnit structures related to Internet Explorer autocomplete. |
| Internet Explorer | `Windows.Data.InternetExplorer.TabRoaming` | `TabRoamingSetting` | vector<SettingUnit> |  | SettingUnit structures related to Internet Explorer tab roaming settings. |
| Lock screen | `Windows.Data.LockScreenSettings` | `kind` | LockScreenKind |  | Specifies whether current Lockscreen is set as Wallpaper or Slideshow or Spotlight |
| Lock screen | `Windows.Data.LockScreenSettings` | `pictureOnSignInScreen` | bool |  | Specifies whether show the lock screen background picture on the sign-in screen is enabled. |
| Lock screen | `Windows.Data.LockScreenSettings` | `funItems` | bool |  | Specifies whether "Get fun facts, tips, tricks, and more on your lock screen" is enabled. |
| Lock screen | `Windows.Data.LockScreenSettings` | `itemId` | wstring |  | The unique ID for the lockscreen wallpaper uploaded to during backup. |
| Lock screen | `Windows.Data.LockScreenSettings` | `contentUri` | wstring |  | The url for the lockscreen wallpaper uploaded to during backup |
| Lock screen | `Windows.Data.LockScreenSettings` | `lockScreenStatus` | wstring |  | Not used. |
| Lock screen | `Windows.Data.LockScreenSettings` | `slideShowAutoLock` |  |  | Not used. |
| Lock screen | `Windows.Data.LockScreenSettings` | `slideShowEnabledOnBattery` |  |  | Not used. |
| Lock screen | `Windows.Data.LockScreenSettings` | `slideshowOptimizePhotoSelection` |  |  | Not used. |
| Lock screen | `Windows.Data.LockScreenSettings` | `slideShowIncludeCameraRoll` |  |  | Not used. |
| Lock screen | `Windows.Data.LockScreenSettings` | `slideShowDuration` |  |  | Not used. |
| Lock screen | `Windows.Data.LockScreenSettings` | `syncRootRelativePaths` |  |  | Not used. |
| Lunar calendar | `Windows.Data.LunarCalendar` | `languageType` | LunarCalendarLanguageType | The default value is Default. | A member of the LunarCalendarLanguageType enumeration. |
| Multiple displays | `Windows.Data.Settings.DisplaySettings.MultipleDisplays` | `rememberWindowLocationsPerMonitorConnection` | nullable<bool> |  | Enables remember window locations per monitor connection |
| Multiple displays | `Windows.Data.Settings.DisplaySettings.MultipleDisplays` | `minimizeWindowsOnMonitorDisconnect` | nullable<bool> |  | Enables minimize windows on monitor disconnect. |
| Multiple displays | `Windows.Data.Settings.DisplaySettings.MultipleDisplays` | `easeCursorMovementBetweenDisplays` | nullable<bool> |  | Enables minimize windows on monitor disconnect. |
| Nightlight | `Windows.Data.BlueLightReduction.ScheduleTime` | `hour` | int8 |  | Hours. |
| Nightlight | `Windows.Data.BlueLightReduction.ScheduleTime` | `minute` | int8 |  | Minutes. |
| Nightlight | `Windows.Data.BlueLightReduction.Settings` | `automaticOnSchedule` | bool |  | Specifies whether blue light reduction is automatically turned on or off based on a schedule. |
| Nightlight | `Windows.Data.BlueLightReduction.Settings` | `automaticOnSunset` | bool |  | Specifies if blue light reduction schedule is automatically set based on sunrise and sunset. |
| Nightlight | `Windows.Data.BlueLightReduction.Settings` | `manualScheduleBlueLightReductionOnTime` | ScheduleTime |  | The start time of blue light reduction for a user manually setting their schedule. |
| Nightlight | `Windows.Data.BlueLightReduction.Settings` | `manualScheduleBlueLightReductionOffTime` | ScheduleTime |  | The end time of blue light reduction for a user manually setting their schedule. |
| Nightlight | `Windows.Data.BlueLightReduction.Settings` | `targetColorTemperature` | int16 |  | The target color temperature (in Kelvin) for blue light reduction. |
| Nightlight | `Windows.Data.BlueLightReduction.Settings` | `sunsetTime` | ScheduleTime |  | The scheduled sunset time for blue light reduction. |
| Nightlight | `Windows.Data.BlueLightReduction.Settings` | `sunriseTime` | ScheduleTime |  | he scheduled sunrise time for blue light reduction. |
| Nightlight | `Windows.Data.BlueLightReduction.Settings` | `previewColorTemperatureChanges` | bool |  | Specifies whether blue light reduction color temperature changes should be previewed. |
| Nightlight | `Windows.Data.BlueLightReduction.Settings` | `darkMode` | bool |  | Specifies whether app mode should change when blue light reduction is turned on or off |
| Nightlight | `Windows.Data.BlueLightReduction.BlueLightReductionState` | `state` | ActiveState |  | The current state of blue light reduction. |
| Nightlight | `Windows.Data.BlueLightReduction.BlueLightReductionState` | `source` | ChangeSource |  | Where the change came from, user change or scheduled change. |
| Nightlight | `Windows.Data.BlueLightReduction.BlueLightReductionState` | `timestampUTC` | int64 |  | The time the change in active state was applied. |
| Nightlight | `Windows.Data.BlueLightReduction.BlueLightReductionState` | `isSupported` | bool |  | Whether or not current configuration supports blue light reduction. |
| NlmSignature | `Windows.Data.Nlm.NlmSignature` | `category` | uint64 |  | Category of signature. |
| Pen and Windows Ink | `Windows.Data.Input.Devices.PenPerDevice` | `singleClickOverride` | uint64 | Any integer value 0 - 13. | Value corresponds to options available for Single Click setting dropdown. |
| Pen and Windows Ink | `Windows.Data.Input.Devices.PenPerDevice` | `singleClickPenWorkspaceVerb` | uint64 | Any integer value 0-3. | Value corresponds to options available for Single Click setting dropdown. |
| Pen and Windows Ink | `Windows.Data.Input.Devices.PenPerDevice` | `doubleClickOverride` | uint64 | Any integer value 0 - 13. | Value corresponds to options available for Double Click setting dropdown. |
| Pen and Windows Ink | `Windows.Data.Input.Devices.PenPerDevice` | `doubleClickPenWorkspaceVerb` | uint64 | Any integer value 0-3. | Value corresponds to options available for Double Click setting dropdown. |
| Pen and Windows Ink | `Windows.Data.Input.Devices.PenPerDevice` | `longPressOverride` | uint64 | Any integer value 0 - 13. | Value corresponds to options available for long press setting dropdown. |
| Pen and Windows Ink | `Windows.Data.Input.Devices.PenPerDevice` | `longPressPenWorkspaceVerb` | uint64 | Any integer value 0-3. | Value corresponds to options available for Long Press setting dropdown. |
| Pen and Windows Ink | `Windows.Data.Input.Devices.PenPerDevice` | `penWorkspaceAppLaunchOnPenDetachEnabled` | bool |  | Specifies whether the pen menu is shown after pen is removed from storage. |
| Pen and Windows Ink | `Windows.Data.Input.Devices.PenPerDevice` | `penEnablePenButtonOverride` | bool |  | Specifies whether apps are allowed to override the shortcut button behavior. |
| Pen and Windows Ink | `Windows.Data.Input.Devices.PenPerDevice` | `singleClickCustomAppPath` | wstring |  | Specifies the path to the app opened on single-click. |
| Pen and Windows Ink | `Windows.Data.Input.Devices.PenPerDevice` | `doubleClickCustomAppPath` | wstring |  | Specifies the path to the app opened on double-click. |
| Pen and Windows Ink | `Windows.Data.Input.Devices.PenPerDevice` | `longPressCustomAppPath` | wstring |  | Specifies the path to the app opened on long-press. |
| Pen and Windows Ink | `Windows.Data.Input.Devices.PenPerDevice` | `singleClickCustomAppID` | wstring |  | Specifies the app ID app opened on single-click. |
| Pen and Windows Ink | `Windows.Data.Input.Devices.PenPerDevice` | `doubleClickCustomAppID` | wstring |  | Specifies the app ID app opened on double-click. |
| Pen and Windows Ink | `Windows.Data.Input.Devices.PenPerDevice` | `longPressCustomAppID` | wstring |  | Specifies the app ID app opened on long-press. |
| Personalization - colors | `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize` | `AppsUseLightTheme` | REG_DWORD | 0/1 | Signifies light/dark color mode for an app. |
| Personalization - colors | `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize` | `SystemUsesLightTheme` | REG_DWORD | 0/1 | Signifies light/dark color mode for Windows. |
| Personalization - colors | `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize` | `EnableTransparency` | REG_DWORD | 0/1 | Signifies the transparency effect on windows and surfaces. |
| Personalization - colors | `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize` | `ColorPrevalance` | REG_DWORD | 0/1 | Signifies the toggle state for "Show Accent Color on Start and Taskbar". |
| Personalization - colors | `HKCU\Control Panel\Desktop` | `AutoColorization` | REG_DWORD | 0/1 | signifies auto-apply accent color based on background or manually. |
| Personalization - colors | `HKCU\Software\Microsoft\Windows\DWM` | `ColorPrevalence` | REG_DWORD | 0/1 | Signifies toggle state for "show Accent color in title bar and windows borders". |
| Personalization - Start - Folders | `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Start` | `VisiblePlaces` | REG_BINARY | A vector of GUIDs. | A list of GUIDs indicating the folders that are shown at the bottom of Start. |
| Personalization - Start - Layout | `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Start` | `Config` | REG_BINARY | A JSON file. | The contents of the Pinned Layout in JSON. |
| Personalization - Start - Show recent apps | `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Start` | `ShowRecentList` | REG_BOOL | 0 or 1 | Specifies whether apps that were recently installed in Start in various surfaces are shown on Start. |
| Personalization - Start - Show recommended apps | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` | `Start_TrackDocs` | REG_BOOL | 0 or 1 | Specifies whether recommended files in Start, recent files in File Explorer, and items in Jump Lists are shown. |
| Personalization - Taskbar - Badges | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarBadges` | `SystemSettings_Taskbar_Badging` | REG_SZ | 0 or 1 | Enables badges for apps on the taskbar. |
| Personalization - Taskbar - Badges | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarBadges` | `SystemSettings_DesktopTaskbar_Badging` | REG_SZ | 0 or 1 | Enables badges for apps on the taskbar. |
| Personalization - Taskbar - Combine buttons, mult-monitor | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\MMTaskbarGlomLevel` | `SystemSettings_DesktopTaskbar_GroupingMode` | REG_SZ | 0, 1, or 2 | 0: Always, 1: When taskbar is full, 2: Never. |
| Personalization - Taskbar - Multi-monitor | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\MMTaskbarEnabled` | `SystemSettings_Taskbar_MultiMon` | REG_SZ | 0 or 1 | Enables showing the taskbar on multiple displays. |
| Personalization - Taskbar - Multi-monitor | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\MMTaskbarEnabled` | `SystemSettings_DesktopTaskbar_MultiMon` | REG_SZ | 0 or 1 | Enables showing the taskbar on multiple displays. |
| Personalization - Taskbar - Multi-monitor taskbar mode | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\MMTaskbarMode` | `SystemSettings_Taskbar_MultiMonTaskbarMode` | REG_SZ | 0, 1, or 2 | 0: Duplicate, 1: Primary and monitor window is on, 2: Monitor window is on. |
| Personalization - Taskbar - Multi-monitor taskbar mode | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\MMTaskbarMode` | `SystemSettings_DesktopTaskbar_MultiMonTaskbarMode` | REG_SZ | 0 1, or 2 | 0: Duplicate, 1: Primary and monitor window is on, 2: Monitor window is on. |
| Personalization - Taskbar - Pinned apps from other devices | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Taskband\FavoritesMigration` | `FavoritesMigration` | REG_BLOB | A binary blob. | This is an opaque binary blob copied from the following location on the backed up. |
| Personalization - Taskbar - Pinned apps from other devices | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Taskband\FavoritesMigration` | `Favorites` | REG_SZ | 0 1 | The format of this key is undocumented. |
| Secondary accounts | `Windows.Data.Account.AccountInfo` | `accountName` | wstring |  | The user name of the account such as "example@outlook.com". |
| Secondary accounts | `Windows.Data.Account.AccountInfo` | `accountId` | wstring |  | The unique identifier of the account. |
| Secondary accounts | `Windows.Data.Account.AccountInfo` | `accountType` | AccountType |  | The type of the account. |
| Secondary accounts | `Windows.Data.Account.AccountInfo` | `country` | wstring |  | The code of the country or region in which a MSA is registered. |
| Secondary accounts | `Windows.Data.Account.AccountInfo` | `safeCustomerId` | wstring |  | An alternative identifier for an MSA. |
| Secondary accounts | `Windows.Data.Account.AccountInfo` | `ageGroup` | wstring | Current values are 0 = unknown, 1 = child, 2 = teen, 3 = adult. | Age group of an MSA, based on the registered birth date of the MSA user. |
| Secondary accounts | `Windows.Data.Account.AccountInfo` | `scope` | wstring |  | Represents the "Sign in options" setting state. |
| Secondary accounts | `Windows.Data.Account.SecondaryAccounts` | `accountDetails` | vector<AccountInfo> |  | A vector of AccountInfo objects representing secondary accounts. |
| USB | `HKCU\Software\Microsoft\Shell\USB` | `NotifyOnUsbErrors` | REG_DWORD | 0 or 1 | Enables notifications for USB errors. |
| USB | `HKCU\Software\Microsoft\Shell\USB` | `NotifyOnWeakCharger` | REG_DWORD | 0 or 1 | Enables notifications for weak charger detected. |
| Windows Backup - Remember my preferences | `Windows.Data.WindowsBackup.Preference.Toggles` | `toggleState` | map<wstring, CloudStorePolicyEnum> |  | The user preference in terms of mapping of sync policy and CloudStorePolicy option. Allowed values for sync policy names are listed below. |
| Windows Update | `HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings` | `IsContinuousInnovationOptedIn` | REG_DWORD | 0/1 | Enables devices to get the latest updates as soon as they're released. |
| Windows Update | `HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings` | `AllowMUUpdateService` | REG_DWORD | 0/1 | Allows users to get other Microsoft products alongside with Windows Updates. |
| Windows Update | `HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings` | `IsExpedited` | REG_DWORD | 0/1 | User selects this to invoke device restart 15 min after all updates finished installing. |
| Windows Update | `HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings` | `RestartNotificationsAllowed2` | REG_DWORD | 0/1 | Users decides if they want to be notified about the updates pending restart. |

### Source: [settings-windows-11](https://learn.microsoft.com/en-us/windows/apps/develop/settings/settings-windows-11)

| Section | Store (key path / type) | Value | Type | Data | Description |
|---|---|---|---|---|---|
| Cellular | `HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\WcmSvc\CellularFailover` | `AllowFailover` | REG_DWORD | 0 or 1 | Use cellular whenever Wi-Fi is poor. |
| Cellular | `HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\WcmSvc\SubscriptionManager\<ICCID>\<IMSI>` | `UserCost` | REG_DWORD | 0 or 1 | Maps the metered/unmetered state to each IMSI. 0: Unmetered. 1: Metered. |
| Cellular | `HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\WwanSvc\DisallowAutoConnectByClient` | `<interface GUID>` | REG_DWORD | 0 or 1 | Let Windows keep the device connected to cellular. |
| Cellular | `HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\WwanSvc\RoamingPolicyForPhone\<interface GUID>` | `InternetAlwaysOn` | REG_DWORD | 0 or 1 | Roaming or no roaming. When entering a roaming area, your data connection will be turned off if roaming is not allowed. |
| Gaming: Game Bar, Game Mode, Gaming Shortcuts | `HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR` | `VKMSaveHistoricalVideo` | REG_DWORD | 0 or 1 | Toggles save historical video enabled. |
| Gaming: Game Bar, Game Mode, Gaming Shortcuts | `HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR` | `VKMTakeScreenshot` | REG_DWORD | 0 or 1 | Toggles take screenshot enabled. |
| Gaming: Game Bar, Game Mode, Gaming Shortcuts | `HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR` | `VKTakeScreenshot` | REG_DWORD | ASCII Value for keys | Key binding for take screenshot. |
| Gaming: Game Bar, Game Mode, Gaming Shortcuts | `HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR` | `VKSaveHistoricalVideo` | REG_DWORD | ASCII Value for keys | Key binding for save historical video. |
| Gaming: Game Bar, Game Mode, Gaming Shortcuts | `HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR` | `VKMToggleBroadcast` | REG_DWORD | 0 or 1 | Toggles broadcast enabled. |
| Gaming: Game Bar, Game Mode, Gaming Shortcuts | `HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR` | `VKToggleBroadcast` | REG_DWORD | ASCII Value for keys | Key binding for take broadcast. |
| Gaming: Game Bar, Game Mode, Gaming Shortcuts | `HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR` | `VKMToggleCameraCapture` | REG_DWORD | 0 or 1 | Toggles camera capture enabled. |
| Gaming: Game Bar, Game Mode, Gaming Shortcuts | `HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR` | `VKToggleCameraCapture` | REG_DWORD | ASCII Value for keys | Key binding for camera capture. |
| Gaming: Game Bar, Game Mode, Gaming Shortcuts | `HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR` | `VKMToggleGameBar` | REG_DWORD | 0 or 1 | Toggles game bar enabled. |
| Gaming: Game Bar, Game Mode, Gaming Shortcuts | `HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR` | `VKToggleGameBar` | REG_DWORD | ASCII Value for keys | Key binding for game bar. |
| Gaming: Game Bar, Game Mode, Gaming Shortcuts | `HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR` | `VKMToggleMicrophoneCapture` | REG_DWORD | 0 or 1 | Toggles microphone capture enabled. |
| Gaming: Game Bar, Game Mode, Gaming Shortcuts | `HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR` | `VKMToggleRecording` | REG_DWORD | 0 or 1 | Toggles recording enabled. |
| Gaming: Game Bar, Game Mode, Gaming Shortcuts | `HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR` | `VKToggleRecording` | REG_DWORD | ASCII Value for keys | Key binding for recording. |
| Gaming: Game Bar, Game Mode, Gaming Shortcuts | `HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR` | `VKMToggleRecordingIndicator` | REG_DWORD | 0 or 1 | Toggles recording indicator enabled. |
| Gaming: Game Bar, Game Mode, Gaming Shortcuts | `HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR` | `VKToggleRecordingIndicator` | REG_DWORD | ASCII Value for keys | Key binding for recording indicator. |
| Gaming: Game Bar, Game Mode, Gaming Shortcuts | `HKCU\Software\Microsoft\GameBar` | `UseNexusForGameBarEnabled` | REG_DWORD | 0 or 1 | Key associated with toggle Gaming -> Game Bar -> "Allow your controller to open Game Bar" |
| Gaming: Game Bar, Game Mode, Gaming Shortcuts | `HKCU\Software\Microsoft\GameBar` | `AutoGameModeEnabled` | REG_DWORD | 0 or 1 | Key associated with toggle Gaming -> Game Mode -> Game Mode. |
| Personalization - Start - Layout - Pins and recommendations | `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` | `Start_Layout` | REG_DWORD | 0 [Default], 1 [More Pins], 2 [More Recommendations] | The start layout. |
| Personalization - Start - Show all pins by default | `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Start` | `ShowAllPinsList` | REG_DWORD | 0 or 1 | Specifies if showing all pins by default |
| Personalization - Start - Recommendations | `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` | `Start_IrisRecommendations` | REG_DWORD | 0 or 1 | Specifies whether recommendations are enabled. |
| Personalization - Taskbar - Alignment | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarAl` | `SystemSettings_DesktopTaskbar_Al` | REG_SZ | 0 or 1 | Specifies the taskbar alignment. 0 is left aligned. 1 is center aligned. |
| Personalization - Taskbar - Autohide | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\StuckRects3\` | `SystemSettings_DesktopTaskbar_Autohide` | REG_BINARY | A binary blob. | This is an opaque binary blob copied from the following location on the backed up. |
| Personalization - Taskbar - Autohide | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\StuckRects3\` | `SystemSettings_Taskbar_Autohide` | REG_BINARY | A binary blob. | This is an opaque binary blob copied from the following location on the backed up. |
| Personalization - Taskbar - Combine buttons | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarGlomLevel` | `SystemSettings_DesktopTaskbar_GroupingMode` | REG_SZ | 0, 1, or 2 | 0: Always, 1: When taskbar is full, 2: Never. |
| Personalization - Taskbar - Flashing | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarFlashing` | `SystemSettings_DesktopTaskbar_Flashing` | REG_SZ | 0 or 1 | Enables flashing for taskbar apps. |
| Personalization - Taskbar - Share window | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarSn` | `SystemSettings_DesktopTaskbar_Sn` | REG_SZ | 0 or 1 | Enables sharing any window from taskbar. |
| Personalization - Taskbar - Show desktop | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarSd` | `SystemSettings_DesktopTaskbar_Sd` | REG_SZ | 0 or 1 | Enables showing the desktop by clicking the far corner of the taskbar. |
| Personalization - Taskbar - Show recent searches | `HKCU\Software\Microsoft\Windows\CurrentVersion\Feeds\DSB\OpenOnHover` | `SystemSettings_DesktopTaskbar_Sh` | REG_SZ | 0 or 1 | Enables showing recent searches when hovering on the search icon in taskbar. |
| Personalization - Taskbar - Task view | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\ShowTaskViewButton` | `SystemSettings_DesktopTaskbar_TaskView` | REG_SZ | 0 or 1 | Specifies whether the task view button is shown on the taskbar. |
| Personalization - Taskbar - Widgets button | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarDa` | `SystemSettings_DesktopTaskbar_Da` | REG_SZ | 0 or 1 | Specifies whether the Widgets button is shown on the taskbar. |
| Personalization - Themes | `Windows.Data.PersonalizationThemes.CurrentThemeType` | `type` | CurrentThemeType |  | The current theme type. |
| Personalization - Themes | `Windows.Data.PersonalizationThemes.CurrentThemeType` | `basePersonalizationThemeName` | wstring |  | The name of the current inbox theme applied in the system. The user may have done customization on top of this inbox theme. |
| Text Input | `HKCU\Software\Microsoft\TabletTip\1.7` | `IsKeyBackgroundEnabled` | REG_DWORD | 0 or 1 | Indicates if key background is enabled. |
| Text Input | `HKCU\Software\Microsoft\TabletTip\1.7` | `KeyLabelSize` | REG_DWORD | Min value: 0, Max value: 2 | Key label size. |
| Text Input | `HKCU\Software\Microsoft\TabletTip\1.7` | `UserKeyboardScalingFactor` | REG_DWORD | Min value: 1, Max value: 1000 | User keyboard scaling factor. |
| Text Input | `HKCU\Software\Microsoft\TabletTip\1.7` | `SelectedThemeIndex` | REG_DWORD | Min value: 0, Max value: 15 | Selected theme index. |
| Text Input | `HKCU\Software\Microsoft\TabletTip\1.7` | `SelectedThemeName` | REG_SZ | One of the following values: "LightTheme", "DarkTheme", "ColorPopTheme", "BlackWhiteTheme", "PoppyRedTheme", "IceBlueTheme", "PlatinumTheme", "TangerineTidesTheme", "LilacRiverTheme", "SilkyDawnTheme", "IndigoBreezeTheme", "PinkBlueTheme", "GreenPurpleTheme", "PinkOrangeTheme", "CustomTheme" | Selected theme name. |
| Text Input | `HKCU\Software\Microsoft\TabletTip\1.7` | `ThemeDataVersion` | REG_DWORD | 2 | Theme data version. |
| Typing | `HKCU\Software\Microsoft\input\Settings` | `EnableAutoShiftEngage` | REG_DWORD | 1 or 0 | Enables auto shift engage. |
| Typing | `HKCU\Software\Microsoft\input\Settings` | `EnableDoubleTapSpace` | REG_DWORD | 1 or 0 | Enables double-tap space. |
| Typing | `HKCU\Software\Microsoft\input\Settings` | `EnableAutocorrection` | REG_DWORD | 1 or 0 | Enables auto-correction. |
| Typing | `HKCU\Software\Microsoft\input\Settings` | `IsVoiceTypingKeyEnabled` | REG_DWORD | 1 or 0 | Indicates if voice typing key is enabled. |
| Typing | `HKCU\Software\Microsoft\input\Settings` | `MultilingualEnabled` | REG_DWORD | 1 or 0 | Indicates if multilingual is enabled. |
| Typing | `HKCU\Software\Microsoft\input\Settings` | `EnableHwkbTextPrediction` | REG_DWORD | An integer. | ASCII Value for keys. |
| VPN | `HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\RasMan\Parameters\Config\VpnCostedNetworkSettings` | `NoCostedNetwork` | REG_DWORD | 0 or 1 | Block VPN over metered networks. |
| VPN | `HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\RasMan\Parameters\Config\VpnCostedNetworkSettings` | `NoRoamingNetwork` | REG_DWORD | 0 or 1 | Block VPN while roaming. |
| Wi-Fi | `HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\WlanSvc\Interfaces\<Interface GUID>` | `RandomMacState` | REG_BINARY | 00 00 00 00 or 01 00 00 00 | Whether to use random hardware addresses for newly configured Wi-Fi networks. |

## Appendix 2B — Complete ms-settings URI Register (from Microsoft Learn)

Source: [Launch the Windows Settings app](https://learn.microsoft.com/en-us/windows/apps/develop/launch/launch-settings-app). Resolution on build 26300.9711 is not yet tested (VP-02).

| # | Category | Settings page | URI | Notes (from source) |
|---|---|---|---|---|
| U001 | Accounts | Access work or school | `ms-settings:workplace` |  |
| U002 | Accounts | Email & app accounts | `ms-settings:emailandaccounts` |  |
| U003 | Accounts | Family & other people | `ms-settings:otherusers` |  |
| U004 | Accounts | Provisioning | `ms-settings:provisioning` | (only available on mobile and if the enterprise has deployed a provisioning package)  (only available if enterprise has deployed a provisioning package) |
| U005 | Accounts | Provisioning | `ms-settings:workplace-provisioning` | (only available on mobile and if the enterprise has deployed a provisioning package)  (only available if enterprise has deployed a provisioning package) |
| U006 | Accounts | Repair token | `ms-settings:workplace-repairtoken` |  |
| U007 | Accounts | Set up a kiosk | `ms-settings:assignedaccess` |  |
| U008 | Accounts | Sign-in options | `ms-settings:signinoptions` |  |
| U009 | Accounts | Sign-in options | `ms-settings:signinoptions-dynamiclock` |  |
| U010 | Accounts | Sync your settings | `ms-settings:sync` | (**Backup page deprecated in Windows 11**) |
| U011 | Accounts | Sync your settings | `ms-settings:backup` | (**Backup page deprecated in Windows 11**) |
| U012 | Accounts | Windows Anywhere | `ms-settings:windowsanywhere` | (device must be Windows Anywhere-capable) |
| U013 | Accounts | Windows Hello setup | `ms-settings:signinoptions-launchfaceenrollment` |  |
| U014 | Accounts | Windows Hello setup | `ms-settings:signinoptions-launchfingerprintenrollment` |  |
| U015 | Accounts | Your info | `ms-settings:yourinfo` |  |
| U016 | Apps | Apps & Features | `ms-settings:appsfeatures` |  |
| U017 | Apps | App features | `ms-settings:appsfeatures-app` | (Reset, manage add-on & downloadable content, etc. for the app) To access this page with a URI, use the  URI and pass an optional parameter of the *package family name* of the app. Example: `` |
| U018 | Apps | App features | `ms-settings:appsfeatures-app` | (Reset, manage add-on & downloadable content, etc. for the app) To access this page with a URI, use the  URI and pass an optional parameter of the *package family name* of the app. Example: `` |
| U019 | Apps | App features | `ms-settings:appsfeatures-app?<PFN>` | (Reset, manage add-on & downloadable content, etc. for the app) To access this page with a URI, use the  URI and pass an optional parameter of the *package family name* of the app. Example: `` |
| U020 | Apps | Apps for websites | `ms-settings:appsforwebsites` |  |
| U021 | Apps | Default apps | `ms-settings:defaultapps` | (**Behavior introduced in Windows 11, version 21H2 (with 2023-04 Cumulative Update) or 22H2 (with 2023-04 Cumulative Update), or later.**) Append the query string parameter in the following formats using the Uri-escaped name of an app to directly launch the default settings page for that app: - regi |
| U022 | Apps | Default browser settings | `ms-settings:defaultbrowsersettings` | (**Deprecated in Windows 11**) |
| U023 | Apps | Manage optional features | `ms-settings:optionalfeatures` |  |
| U024 | Apps | Offline Maps | `ms-settings:maps` | (Download maps) |
| U025 | Apps | Offline Maps | `ms-settings:maps-downloadmaps` | (Download maps) |
| U026 | Apps | Startup apps | `ms-settings:startupapps` |  |
| U027 | Apps | Video playback | `ms-settings:videoplayback` |  |
| U028 | Control Center | Control center | `ms-settings:controlcenter` |  |
| U029 | Cortana | Cortana across my devices | `ms-settings:cortana-notifications` |  |
| U030 | Cortana | More details | `ms-settings:cortana-moredetails` |  |
| U031 | Cortana | Permissions & History | `ms-settings:cortana-permissions` |  |
| U032 | Cortana | Searching Windows | `ms-settings:cortana-windowssearch` |  |
| U033 | Cortana | Talk to Cortana | `ms-settings:cortana-language` |  |
| U034 | Cortana | Talk to Cortana | `ms-settings:cortana` |  |
| U035 | Cortana | Talk to Cortana | `ms-settings:cortana-talktocortana` |  |
| U036 | Devices | AutoPlay | `ms-settings:autoplay` |  |
| U037 | Devices | Bluetooth | `ms-settings:bluetooth` |  |
| U038 | Devices | Connected Devices | `ms-settings:connecteddevices` |  |
| U039 | Devices | Default camera | `ms-settings:camera` | (**Behavior deprecated in Windows 10, version 1809 and later**) |
| U040 | Devices | Camera settings | `ms-settings:camera` | (**Behavior introduced in Windows 11, build 22000 and later**) Append the query string parameter `cameraId` set to the Uri-escaped symbolic link name of a camera device to directly launch the settings for that camera. For more information, see Launch the camera settings page. |
| U041 | Devices | Mouse & touchpad | `ms-settings:mousetouchpad` | (touchpad settings only available on devices that have a touchpad) |
| U042 | Devices | Pen & Windows Ink | `ms-settings:pen` |  |
| U043 | Devices | Printers & scanners | `ms-settings:printers` |  |
| U044 | Devices | Touch | `ms-settings:devices-touch` |  |
| U045 | Devices | Touchpad | `ms-settings:devices-touchpad` | (only available if touchpad hardware is present) |
| U046 | Devices | Text Suggestions | `ms-settings:devicestyping-hwkbtextsuggestions` |  |
| U047 | Devices | Typing | `ms-settings:typing` |  |
| U048 | Devices | USB | `ms-settings:usb` |  |
| U049 | Devices | Wheel | `ms-settings:wheel` | (only available if a **Surface Dial** device is paired) |
| U050 | Devices | Your phone | `ms-settings:mobile-devices` |  |
| U051 | Ease of access | Audio | `ms-settings:easeofaccess-audio` |  |
| U052 | Ease of access | Closed captions | `ms-settings:easeofaccess-closedcaptioning` |  |
| U053 | Ease of access | Color filters | `ms-settings:easeofaccess-colorfilter` |  |
| U054 | Ease of access | Color filters | `ms-settings:easeofaccess-colorfilter-adaptivecolorlink` |  |
| U055 | Ease of access | Color filters | `ms-settings:easeofaccess-colorfilter-bluelightlink` |  |
| U056 | Ease of access | Display | `ms-settings:easeofaccess-display` |  |
| U057 | Ease of access | Eye control | `ms-settings:easeofaccess-eyecontrol` |  |
| U058 | Ease of access | Hearing devices | `ms-settings:easeofaccess-hearingaids` | (**Added in Windows 11, Version 24H2**) |
| U059 | Ease of access | High contrast | `ms-settings:easeofaccess-highcontrast` |  |
| U060 | Ease of access | Keyboard | `ms-settings:easeofaccess-keyboard` |  |
| U061 | Ease of access | Magnifier | `ms-settings:easeofaccess-magnifier` |  |
| U062 | Ease of access | Mouse | `ms-settings:easeofaccess-mouse` |  |
| U063 | Ease of access | Mouse pointer & touch | `ms-settings:easeofaccess-mousepointer` |  |
| U064 | Ease of access | Narrator | `ms-settings:easeofaccess-narrator` |  |
| U065 | Ease of access | Narrator | `ms-settings:easeofaccess-narrator-isautostartenabled` |  |
| U066 | Ease of access | Speech | `ms-settings:easeofaccess-speechrecognition` |  |
| U067 | Ease of access | Text cursor | `ms-settings:easeofaccess-cursor` |  |
| U068 | Ease of access | Visual Effects | `ms-settings:easeofaccess-visualeffects` |  |
| U069 | Extras | Extras | `ms-settings:extras` | (only available if "settings apps" have been installed, for example, by a 3rd party) |
| U070 | Family Group | Family Group | `ms-settings:family-group` |  |
| U071 | Gaming | Game bar | `ms-settings:gaming-gamebar` |  |
| U072 | Gaming | Game DVR | `ms-settings:gaming-gamedvr` |  |
| U073 | Gaming | Game Mode | `ms-settings:gaming-gamemode` |  |
| U074 | Gaming | Playing a game full screen | `ms-settings:quietmomentsgame` |  |
| U075 | Gaming | TruePlay | `ms-settings:gaming-trueplay` | (**As of Windows 10, version 1809 (10.0; Build 17763), this feature is removed from Windows**) |
| U076 | Mixed reality | Audio and speech | `ms-settings:holographic-audio` |  |
| U077 | Mixed reality | Environment | `ms-settings:privacy-holographic-environment` |  |
| U078 | Mixed reality | Headset display | `ms-settings:holographic-headset` |  |
| U079 | Mixed reality | Uninstall | `ms-settings:holographic-management` |  |
| U080 | Mixed reality | Startup and desktop | `ms-settings:holographic-startupandesktop` |  |
| U081 | Network and internet | Network & internet | `ms-settings:network-status` |  |
| U082 | Network and internet | Advanced settings | `ms-settings:network-advancedsettings` |  |
| U083 | Network and internet | Airplane mode | `ms-settings:network-airplanemode` |  |
| U084 | Network and internet | Airplane mode | `ms-settings:proximity` |  |
| U085 | Network and internet | Cellular & SIM | `ms-settings:network-cellular` |  |
| U086 | Network and internet | Dial-up | `ms-settings:network-dialup` |  |
| U087 | Network and internet | DirectAccess | `ms-settings:network-directaccess` | (only available if DirectAccess is enabled) |
| U088 | Network and internet | Ethernet | `ms-settings:network-ethernet` |  |
| U089 | Network and internet | Manage known networks | `ms-settings:network-wifisettings` |  |
| U090 | Network and internet | Mobile hotspot | `ms-settings:network-mobilehotspot` |  |
| U091 | Network and internet | Proxy | `ms-settings:network-proxy` |  |
| U092 | Network and internet | VPN | `ms-settings:network-vpn` |  |
| U093 | Network and internet | Wi-Fi | `ms-settings:network-wifi` | (only available if the device has a wifi adapter) |
| U094 | Network and internet | Wi-Fi provisioning | `ms-settings:wifi-provisioning` |  |
| U095 | Personalization | Background | `ms-settings:personalization-background` |  |
| U096 | Personalization | Choose which folders appear on Start | `ms-settings:personalization-start-places` |  |
| U097 | Personalization | Colors | `ms-settings:personalization-colors` |  |
| U098 | Personalization | Colors | `ms-settings:colors` |  |
| U099 | Personalization | Customize Copilot key on keyboard | `ms-settings:personalization-textinput-copilot-hardwarekey` |  |
| U100 | Personalization | Dynamic Lighting | `ms-settings:personalization-lighting` |  |
| U101 | Personalization | Fonts | `ms-settings:fonts` |  |
| U102 | Personalization | Glance | `ms-settings:personalization-glance` | (**Deprecated in Windows 10, version 1809 and later**) |
| U103 | Personalization | Lock screen | `ms-settings:lockscreen` |  |
| U104 | Personalization | Navigation bar | `ms-settings:personalization-navbar` | (**Deprecated in Windows 10, version 1809 and later**) |
| U105 | Personalization | Personalization (category) | `ms-settings:personalization` |  |
| U106 | Personalization | Start | `ms-settings:personalization-start` |  |
| U107 | Personalization | Taskbar | `ms-settings:taskbar` |  |
| U108 | Personalization | Text input | `ms-settings:personalization-textinput` |  |
| U109 | Personalization | Touch Keyboard | `ms-settings:personalization-touchkeyboard` |  |
| U110 | Personalization | Themes | `ms-settings:themes` |  |
| U111 | Phone | Your phone | `ms-settings:mobile-devices` | (Opens **Your Phone** app) |
| U112 | Phone | Your phone | `ms-settings:mobile-devices-addphone` | (Opens **Your Phone** app) |
| U113 | Phone | Your phone | `ms-settings:mobile-devices-addphone-direct` | (Opens **Your Phone** app) |
| U114 | Phone | Device Usage | `ms-settings:deviceusage` |  |
| U115 | Privacy | Accessory apps | `ms-settings:privacy-accessoryapps` | (**Deprecated in Windows 10, version 1809 and later**) |
| U116 | Privacy | Account info | `ms-settings:privacy-accountinfo` |  |
| U117 | Privacy | Activity history | `ms-settings:privacy-activityhistory` |  |
| U118 | Privacy | Advertising ID | `ms-settings:privacy-advertisingid` | (**Deprecated in Windows 10, version 1809 and later**) |
| U119 | Privacy | App diagnostics | `ms-settings:privacy-appdiagnostics` |  |
| U120 | Privacy | Automatic file downloads | `ms-settings:privacy-automaticfiledownloads` |  |
| U121 | Privacy | Background Apps | `ms-settings:privacy-backgroundapps` | (**Deprecated in Windows 11, 21H2 and later**) **Note:** In Windows 11, the background app permissions are accessed individually. To view the permissions, go to **Apps->Installed apps** and then select **"..."** on a modern app and choose **Advanced options**. The advanced page is present for modern |
| U122 | Privacy | Background Apps | `ms-settings:appsfeatures-app` | (**Deprecated in Windows 11, 21H2 and later**) **Note:** In Windows 11, the background app permissions are accessed individually. To view the permissions, go to **Apps->Installed apps** and then select **"..."** on a modern app and choose **Advanced options**. The advanced page is present for modern |
| U123 | Privacy | Background Spatial Perception | `ms-settings:privacy-backgroundspatialperception` |  |
| U124 | Privacy | Calendar | `ms-settings:privacy-calendar` |  |
| U125 | Privacy | Call history | `ms-settings:privacy-callhistory` |  |
| U126 | Privacy | Camera | `ms-settings:privacy-webcam` |  |
| U127 | Privacy | Contacts | `ms-settings:privacy-contacts` |  |
| U128 | Privacy | Documents | `ms-settings:privacy-documents` |  |
| U129 | Privacy | Downloads folder | `ms-settings:privacy-downloadsfolder` |  |
| U130 | Privacy | Email | `ms-settings:privacy-email` |  |
| U131 | Privacy | Eye tracker | `ms-settings:privacy-eyetracker` | (requires eyetracker hardware) |
| U132 | Privacy | Feedback & diagnostics | `ms-settings:privacy-feedback` |  |
| U133 | Privacy | File system | `ms-settings:privacy-broadfilesystemaccess` |  |
| U134 | Privacy | General | `ms-settings:privacy` | or |
| U135 | Privacy | General | `ms-settings:privacy-general` | or |
| U136 | Privacy | Graphics | `ms-settings:privacy-graphicscaptureprogrammatic` |  |
| U137 | Privacy | Graphics | `ms-settings:privacy-graphicscapturewithoutborder` |  |
| U138 | Privacy | Inking & typing | `ms-settings:privacy-speechtyping` |  |
| U139 | Privacy | Location | `ms-settings:privacy-location` |  |
| U140 | Privacy | Messaging | `ms-settings:privacy-messaging` |  |
| U141 | Privacy | Microphone | `ms-settings:privacy-microphone` |  |
| U142 | Privacy | Motion | `ms-settings:privacy-motion` |  |
| U143 | Privacy | Music Library | `ms-settings:privacy-musiclibrary` |  |
| U144 | Privacy | Notifications | `ms-settings:privacy-notifications` |  |
| U145 | Privacy | Other devices | `ms-settings:privacy-customdevices` |  |
| U146 | Privacy | Phone calls | `ms-settings:privacy-phonecalls` |  |
| U147 | Privacy | Pictures | `ms-settings:privacy-pictures` |  |
| U148 | Privacy | Radios | `ms-settings:privacy-radios` |  |
| U149 | Privacy | Speech | `ms-settings:privacy-speech` |  |
| U150 | Privacy | Tasks | `ms-settings:privacy-tasks` |  |
| U151 | Privacy | Videos | `ms-settings:privacy-videos` |  |
| U152 | Privacy | Voice activation | `ms-settings:privacy-voiceactivation` |  |
| U153 | Search | Search | `ms-settings:search` |  |
| U154 | Search | Search more details | `ms-settings:search-moredetails` |  |
| U155 | Search | Search Permissions | `ms-settings:search-permissions` |  |
| U156 | Sound | Volume mixer | `ms-settings:apps-volume` |  |
| U157 | Sound | Sound | `ms-settings:sound` |  |
| U158 | Sound | Sound devices | `ms-settings:sound-devices` |  |
| U159 | Sound | Default microphone | `ms-settings:sound-defaultinputproperties` |  |
| U160 | Sound | Default audio output | `ms-settings:sound-defaultoutputproperties` |  |
| U161 | Sound | Audio device properties (specific device) | `ms-settings:sound-properties?endpointId={0.0.0.00000000}.{aaaaaaaa-0000-1111-2222-bbbbbbbbbbbb}` | {0.0.0.00000000}.{aaaaaaaa-0000-1111-2222-bbbbbbbbbbbb} **Note:** User of URI must know the `endpointId` string to use. |
| U162 | Sound | Audio device properties (specific device) | `ms-settings:sound-properties?interfaceId=\\?\SWD#MMDEVAPI#{3.0.0.00000003}.{bbbbbbbb-1111-2222-3333-cccccccccccc}#{cccccccc-2222-3333-4444-dddddddddddd}` | \\?\SWD#MMDEVAPI#{3.0.0.00000003}.{bbbbbbbb-1111-2222-3333-cccccccccccc}#{cccccccc-2222-3333-4444-dddddddddddd} **Note:** User of URI must know the `interfaceId` string to use and the string must be escaped correctly before sending. |
| U163 | Surface Hub | Accounts | `ms-settings:surfacehub-accounts` |  |
| U164 | Surface Hub | Session cleanup | `ms-settings:surfacehub-sessioncleanup` |  |
| U165 | Surface Hub | Team Conferencing | `ms-settings:surfacehub-calling` |  |
| U166 | Surface Hub | Team device management | `ms-settings:surfacehub-devicemanagement` |  |
| U167 | Surface Hub | Welcome screen | `ms-settings:surfacehub-welcome` |  |
| U168 | System | About | `ms-settings:about` |  |
| U169 | System | Advanced display settings | `ms-settings:display-advanced` | (only available on devices that support advanced display options) |
| U170 | System | Battery Saver | `ms-settings:batterysaver` | (only available on devices that have a battery, such as a tablet) |
| U171 | System | Battery Saver settings | `ms-settings:batterysaver-settings` | (only available on devices that have a battery, such as a tablet) |
| U172 | System | Battery use | `ms-settings:batterysaver-usagedetails` | (only available on devices that have a battery, such as a tablet) |
| U173 | System | Clipboard | `ms-settings:clipboard` |  |
| U174 | System | Default Save Locations | `ms-settings:savelocations` |  |
| U175 | System | Display | `ms-settings:display` |  |
| U176 | System | Display | `ms-settings:screenrotation` |  |
| U177 | System | Duplicating my display | `ms-settings:quietmomentspresentation` |  |
| U178 | System | During these hours | `ms-settings:quietmomentsscheduled` |  |
| U179 | System | Encryption | `ms-settings:deviceencryption` |  |
| U180 | System | Energy recommendations | `ms-settings:energyrecommendations` | **(Available in February Moment update for Windows 11, Version 22H2, Build 22624 or later)** |
| U181 | System | Focus assist | `ms-settings:quiethours` |  |
| U182 | System | Graphics Settings | `ms-settings:display-advancedgraphics` | (only available on devices that support advanced graphics options) |
| U183 | System | Graphics Default Settings | `ms-settings:display-advancedgraphics-default` |  |
| U184 | System | Multitasking | `ms-settings:multitasking` |  |
| U185 | System | Multitasking | `ms-settings:multitasking-sgupdate` |  |
| U186 | System | Night light settings | `ms-settings:nightlight` |  |
| U187 | System | Projecting to this PC | `ms-settings:project` |  |
| U188 | System | Shared experiences | `ms-settings:crossdevice` |  |
| U189 | System | Tablet mode | `ms-settings:tabletmode` | (**Removed in Windows 11**) |
| U190 | System | Taskbar | `ms-settings:taskbar` |  |
| U191 | System | Notifications & actions | `ms-settings:notifications` |  |
| U192 | System | Remote Desktop | `ms-settings:remotedesktop` |  |
| U193 | System | Phone | `ms-settings:phone` | (**Deprecated in Windows 10, version 1809 and later**) |
| U194 | System | Power & sleep | `ms-settings:powersleep` |  |
| U195 | System | Presence sensing | `ms-settings:presence` | (**Added in May Moment update for Windows 11, Version 22H2, Build 22624**) |
| U196 | System | Storage | `ms-settings:storagesense` |  |
| U197 | System | Storage Sense | `ms-settings:storagepolicies` |  |
| U198 | System | Storage recommendations | `ms-settings:storagerecommendations` |  |
| U199 | System | Disks & volumes | `ms-settings:disksandvolumes` |  |
| U200 | Time and language | Date & time | `ms-settings:dateandtime` |  |
| U201 | Time and language | Japan IME settings | `ms-settings:regionlanguage-jpnime` | (available if the Microsoft Japan input method editor is installed) |
| U202 | Time and language | Region | `ms-settings:regionformatting` |  |
| U203 | Time and language | Language | `ms-settings:keyboard` |  |
| U204 | Time and language | Language | `ms-settings:keyboard-advanced` |  |
| U205 | Time and language | Language | `ms-settings:regionlanguage` |  |
| U206 | Time and language | Language | `ms-settings:regionlanguage-bpmfime` |  |
| U207 | Time and language | Language | `ms-settings:regionlanguage-cangjieime` |  |
| U208 | Time and language | Language | `ms-settings:regionlanguage-chsime-wubi-udp` |  |
| U209 | Time and language | Language | `ms-settings:regionlanguage-quickime` |  |
| U210 | Time and language | Language | `ms-settings:regionlanguage-korime` |  |
| U211 | Time and language | Pinyin IME settings | `ms-settings:regionlanguage-chsime-pinyin` | (available if the Microsoft Pinyin input method editor is installed) |
| U212 | Time and language | Pinyin IME settings | `ms-settings:regionlanguage-chsime-pinyin-domainlexicon` | (available if the Microsoft Pinyin input method editor is installed) |
| U213 | Time and language | Pinyin IME settings | `ms-settings:regionlanguage-chsime-pinyin-keyconfig` | (available if the Microsoft Pinyin input method editor is installed) |
| U214 | Time and language | Pinyin IME settings | `ms-settings:regionlanguage-chsime-pinyin-udp` | (available if the Microsoft Pinyin input method editor is installed) |
| U215 | Time and language | Speech | `ms-settings:speech` |  |
| U216 | Time and language | Wubi IME settings | `ms-settings:regionlanguage-chsime-wubi` | (available if the Microsoft Wubi input method editor is installed) |
| U217 | Update and security | Activation | `ms-settings:activation` |  |
| U218 | Update and security | Backup | `ms-settings:backup` | (page removed in Windows 11; opens Sync) |
| U219 | Update and security | Delivery Optimization | `ms-settings:delivery-optimization` |  |
| U220 | Update and security | Delivery Optimization | `ms-settings:delivery-optimization-activity` |  |
| U221 | Update and security | Delivery Optimization | `ms-settings:delivery-optimization-advanced` |  |
| U222 | Update and security | Find My Device | `ms-settings:findmydevice` |  |
| U223 | Update and security | For developers | `ms-settings:developers` |  |
| U224 | Update and security | Recovery | `ms-settings:recovery` |  |
| U225 | Update and security | Launch Security Key Enrollment | `ms-settings:signinoptions-launchsecuritykeyenrollment` |  |
| U226 | Update and security | Troubleshoot | `ms-settings:troubleshoot` |  |
| U227 | Update and security | Windows Security | `ms-settings:windowsdefender` |  |
| U228 | Update and security | Windows Insider Program | `ms-settings:windowsinsider` | (only present if user is enrolled in WIP) |
| U229 | Update and security | Windows Insider Program | `ms-settings:windowsinsider-optin` | (only present if user is enrolled in WIP) |
| U230 | Update and security | Windows Update | `ms-settings:windowsupdate` |  |
| U231 | Update and security | Windows Update | `ms-settings:windowsupdate-action` |  |
| U232 | Update and security | Windows Update-Active hours | `ms-settings:windowsupdate-activehours` |  |
| U233 | Update and security | Windows Update-Advanced options | `ms-settings:windowsupdate-options` |  |
| U234 | Update and security | Windows Update-Optional updates | `ms-settings:windowsupdate-optionalupdates` |  |
| U235 | Update and security | Windows Update-Restart options | `ms-settings:windowsupdate-restartoptions` |  |
| U236 | Update and security | Windows Update-Seeker on demand | `ms-settings:windowsupdate-seekerondemand` |  |
| U237 | Update and security | Windows Update-View update history | `ms-settings:windowsupdate-history` |  |

---

## Resume Point

Part 3 starts with Batch B2, item WA.01.01 (Windows 11 desktop: wallpaper/slideshow/Spotlight). The live-build validation of Chapter 11 is tracked as VP-01/VP-02.