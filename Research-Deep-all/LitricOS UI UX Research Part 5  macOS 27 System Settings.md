# LitricOS UI/UX Research Encyclopedia — Part 5 of N

Chapter 13: macOS 27 System Settings inventory (Batch B4)

| Field | Value |
|---|---|
| Document ID | LITRIC-RES-P05 |
| Revision | 1.0 |
| Target release | macOS 27.0.1 "Golden Gate", build 26A434 (Apple silicon only); Mac User Guide 27 (`/27/mac/27` topic URLs) |
| Access date | 2026-10-25 |
| Checklist scope | Research D (`MD`): 38 leaf items |
| Official extraction | 114 Apple pages fetched in full (111 with content, 3 redirected to the guide home); 1,377 control rows (`MD.xx.cNNN`) |
| Independent extraction | 29 non-Apple pages read in full; 251 claim rows (§13.40) |
| URL register | 209 `x-apple.systempreferences` rows in `macos_settings_urls.csv` |
| Status (MD leaves) | 1 COMPLETE, 37 PARTIAL |
| Companion files | `macos_settings_urls.csv`, `litricos_coverage_checklist.csv` (rev 1.4), `litricos_source_register.csv` (rev 1.4), `md_raw.json`, `md_raw2.json` |
| Resume point | Part 6 → Batch B5 (MC: macOS 27 desktop model), starting at MC.01 |

---

## Master Index (updated)

| Ch. | Title | Status |
|---|---|---|
| 0–6 | Baseline, checklist, plan (Part 1) | Published |
| 10 | Windows 11 shell and system UI (WA) | PARTIAL (Part 3) |
| 11 | Windows 11 26H2 Settings catalogue (WB) | PARTIAL (Part 2) |
| 12 | Fluent / WinUI design system (WC) | 2 COMPLETE, 11 PARTIAL (Part 4) |
| 13 | macOS 27 System Settings (MD) | 1 COMPLETE, 37 PARTIAL (this part) |
| 14 | macOS 27 desktop model (MC) | Next (Part 6) |
| 15–45 | Remaining chapters | PENDING |

---

## 13.0 Method, Status Rules and Limitations

### 13.0.1 Method

1. Topic discovery. Apple's Mac User Guide 27 table of contents ([guide TOC](https://support.apple.com/guide/mac-help/toc/27)) does not list the per-pane settings-reference topics. Each of 123 targets was therefore found by a search restricted to support.apple.com, and the topic ID was normalised to the canonical `/<topic>/27/mac/27` form.
2. Full-page extraction. 114 Apple pages were fetched in full. A structured extraction recorded every control with its section, exact label, control type, options, default (only if stated), description, conditions and effects. It was told to record only what the page states.
3. Independent sources. 12 queries excluding apple.com produced 96 hits. 29 non-Apple pages were read in full to find macOS 27 changes and contradictions. Unfetched hits are SNIPPET_ONLY in the register.
4. URL schemes and MDM. Apple's device-management reference pages and 8 community lists of pane identifiers were read in full and merged into `macos_settings_urls.csv`.

### 13.0.2 Status rules and tags

- A leaf is COMPLETE only when its controls, defaults and conditions are evidenced, no contradiction remains open, and no live check is pending. No pane met this bar without a live walk, so pane leaves are PARTIAL.
- Tiers: T1 = Apple documentation; T2 = source code or repository; T4 = independent article; T5 = forum.
- `Default` shows "—" unless the page states a default. Apple's guide almost never states one.
- Control types (toggle, popup, slider, button, etc.) come from the page wording. When the page does not name the widget, the type was inferred and is therefore lower confidence.

### 13.0.3 Limitations

- The guide's `/27/` pages include text inherited from earlier releases. For example, the About page still describes "Graphics (Intel only)" and Intel processor labels ([About settings](https://support.apple.com/guide/mac-help/mchlea7173f3/27/mac/27)), even though macOS 27 supports only Apple silicon. Labels in the guide may therefore lag the shipping UI (see §13.41).
- No live Mac was available. Exact sidebar order, defaults and pixel geometry are deferred to VP-13–VP-17.
- Apple protects all screenshots, icons and wallpapers. None were copied (Rule 10).

---

## 13.1 MD.01 Sidebar enumeration

Apple says settings "are organized into categories" in the sidebar and "may vary depending on your Mac and the apps you installed" ([Customize your Mac with System Settings](https://support.apple.com/guide/mac-help/mh15217/27/mac/27)). A red badge on the System Settings Dock icon means that one or more actions are needed (same page). Search covers settings, Suggestions and Recents ([Find options in System Settings](https://support.apple.com/guide/mac-help/mchl8d10839d/27/mac/27)). There is also a back control for previously viewed settings ([Go back to settings you viewed](https://support.apple.com/guide/mac-help/mchldfdf4f86/27/mac/27)).

Apple does not publish the sidebar order. The list below gives every top-level pane that a macOS 27 guide page reaches by navigation path. It follows the checklist order, not the on-screen order (GAP-B4-01). The Sidebar.plist inside the app bundle is the authoritative order source ([jaywcjlove README](https://github.com/jaywcjlove/SystemSettings-URLs-macOS/blob/main/README.md), T2). Reading it is test VP-13.

| ID | Pane (27 guide label) | Hardware / condition | Evidence |
|---|---|---|---|
| MD.01.p01 | [your name] (Apple Account) | Shown at the top of the sidebar | [Apple Account](https://support.apple.com/guide/mac-help/mchl3f671010/27/mac/27) |
| MD.01.p02 | Family | Family Sharing | [Family](https://support.apple.com/guide/mac-help/mchl37e74c6a/27/mac/27) |
| MD.01.p03 | Wi-Fi | Wi-Fi hardware | [Wi-Fi](https://support.apple.com/guide/mac-help/mchl339ee00e/27/mac/27) |
| MD.01.p04 | Bluetooth | — | [Bluetooth](https://support.apple.com/guide/mac-help/blth8111/27/mac/27) |
| MD.01.p05 | Network | — | [Network](https://support.apple.com/guide/mac-help/mchlp1102/27/mac/27) |
| MD.01.p06 | VPN | Shown when a VPN configuration exists | [VPN](https://support.apple.com/guide/mac-help/mchlp1579/27/mac/27) |
| MD.01.p07 | Battery | Mac laptops | [Battery](https://support.apple.com/guide/mac-help/mchlfc3b7879/27/mac/27) |
| MD.01.p08 | Energy | Mac desktops | [Energy](https://support.apple.com/guide/mac-help/mchlp1168/27/mac/27) |
| MD.01.p09 | General | — | §13.9 |
| MD.01.p10 | Accessibility | — | [Accessibility](https://support.apple.com/guide/mac-help/mchlp1400/27/mac/27) |
| MD.01.p11 | Appearance | — | [Appearance](https://support.apple.com/guide/mac-help/mchlp1225/27/mac/27) |
| MD.01.p12 | Siri | Guide label; checklist used "Apple Intelligence & Siri" | [Siri settings](https://support.apple.com/guide/mac-help/mchl3fd7fc15/27/mac/27) |
| MD.01.p13 | Control Center | Guide path: menu bar > Control Center > Edit Controls | [Control Center](https://support.apple.com/guide/mac-help/mchlc9d0e1f2/27/mac/27) |
| MD.01.p14 | Desktop & Dock | — | [Desktop & Dock](https://support.apple.com/guide/mac-help/mchlp1119/27/mac/27) |
| MD.01.p15 | Displays | — | [Displays](https://support.apple.com/guide/mac-help/mh40768/27/mac/27) |
| MD.01.p16 | Menu Bar | New separate pane in 27 | [Menu Bar](https://support.apple.com/guide/mac-help/mchlad96d366/27/mac/27) |
| MD.01.p17 | Spotlight | — | [Spotlight](https://support.apple.com/guide/mac-help/mchl54d95e8a/27/mac/27) |
| MD.01.p18 | Wallpaper | — | [Wallpaper](https://support.apple.com/guide/mac-help/mchlp1103/27/mac/27) |
| MD.01.p19 | Screen Saver | Guide path `System Settings > Screen Saver` | [Screen Saver](https://support.apple.com/guide/mac-help/mchlp1227/27/mac/27) |
| MD.01.p20 | Notifications | — | [Notifications](https://support.apple.com/guide/mac-help/mchl205da693/27/mac/27) |
| MD.01.p21 | Sound | — | [Sound](https://support.apple.com/guide/mac-help/mchl9777ee30/27/mac/27) |
| MD.01.p22 | Focus | — | [Focus](https://support.apple.com/guide/mac-help/mchlff5da36d/27/mac/27) |
| MD.01.p23 | Screen Time | — | [Screen Time](https://support.apple.com/guide/mac-help/mchlca5139d2/27/mac/27) |
| MD.01.p24 | Lock Screen | — | [Lock Screen](https://support.apple.com/guide/mac-help/mh11784/27/mac/27) |
| MD.01.p25 | Privacy & Security | — | [Privacy & Security](https://support.apple.com/guide/mac-help/mchl211c911f/27/mac/27) |
| MD.01.p26 | Touch ID & Password | Macs with Touch ID | [Touch ID & Password](https://support.apple.com/guide/mac-help/mchl319754b3/27/mac/27) |
| MD.01.p27 | Login Password | Macs without Touch ID | [Login Password](https://support.apple.com/guide/mac-help/mchl0daacc9f/27/mac/27) |
| MD.01.p28 | Users & Groups | — | [Users & Groups](https://support.apple.com/guide/mac-help/mtusr001/27/mac/27) |
| MD.01.p29 | Internet Accounts | — | [Internet Accounts](https://support.apple.com/guide/mac-help/mh35705/27/mac/27) |
| MD.01.p30 | Game Center | — | [Game Center (Games guide)](https://support.apple.com/en-bw/guide/games/mchle962c097/1.0/mac/27) |
| MD.01.p31 | Wallet & Apple Pay | — | [Wallet & Apple Pay](https://support.apple.com/guide/mac-help/mchl16329b92/27/mac/27) |
| MD.01.p32 | Keyboard | — | [Keyboard](https://support.apple.com/guide/mac-help/kbdm162/27/mac/27) |
| MD.01.p33 | Mouse | — | [Mouse](https://support.apple.com/guide/mac-help/mh29222/27/mac/27) |
| MD.01.p34 | Trackpad | — | [Trackpad](https://support.apple.com/guide/mac-help/mchlp1226/27/mac/27) |
| MD.01.p35 | Game Controllers | Games guide path `System Settings > Game Controllers` | [Games guide](https://support.apple.com/guide/games/connect-a-game-controller-devf8cec167c/mac) |
| MD.01.p36 | Printers & Scanners | — | [Printers & Scanners](https://support.apple.com/guide/mac-help/prtct004/27/mac/27) |
| MD.01.p37 | [AirPods name] | Shown while AirPods are connected | [Use AirPods with your Mac](https://support.apple.com/guide/mac-help/mchld2f124d1/27/mac/27) |
| MD.01.p38 | Device Management | Guide path "Apple menu > Device Management settings" | [Device Management](https://support.apple.com/guide/mac-help/mh35474/27/mac/27) |

Not found as 27 sidebar items: Passwords (legacy topic `sfri40599` redirects to the guide home) and Ink (no 27 topic).

Independent reports on the window itself: Michael Tsai says System Settings can be resized in any direction, made narrower than its default size, and given a wider sidebar, but that this capability "is hidden behind a feature flag" ([Michael Tsai](https://mjtsai.com/blog/2026/09/14/macos-27-golden-gate/)). This is T4 and unverified.


## 13.2 MD.02 Apple Account

Status: **PARTIAL**. Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13).


### Apple Account: Manage your Apple Account settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl3f671010/27/mac/27) · topic `mchl3f671010` · path: System Settings > your name at the top of the sidebar · "Open … for me" link: yes

Use Apple Account settings to manage personal information, sign-in and security, payment and shipping, iCloud, Family Sharing, media and purchases, trusted devices, and other account settings.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.02.c001 | Apple Account settings | your name | other | — | — | Click your name at the top of the sidebar to open Apple Account settings. | If your name isn’t shown, click “Sign in” instead. |
| MD.02.c002 | Apple Account settings | Sign in | other | — | — | Click to sign in to your Apple Account. | Shown if your name isn’t at the top of the sidebar. |
| MD.02.c003 | Apple Account settings | Apple Account email address or phone number | field | — | — | Enter your Apple Account email address or phone number when signing in. | When signing in. |
| MD.02.c004 | Apple Account settings | password | field | — | — | Enter your password when signing in. | When signing in. |
| MD.02.c005 | Apple Account settings | Open Apple Account settings for me | link | — | — | Opens Apple Account settings. | — |
| MD.02.c006 | Apple Account settings | Apple Account picture | other | Photo, Memoji, emoji, or other image | — | Use these options to set an image as your Apple Account picture. | — |
| MD.02.c007 | Apple Account settings | Personal Information | other | — | — | Use these options to change the name and birthday associated with your Apple Account. | — |
| MD.02.c008 | Apple Account settings | Sign-In & Security | other | — | — | Use these options to change the contact information, password, trusted phone numbers, and security settings associated with your Apple Account. | — |
| MD.02.c009 | Apple Account settings | Payment & Shipping | other | — | — | Use these options to see the payment method or update the shipping address associated with your Apple Account. | — |
| MD.02.c010 | Apple Account settings | iCloud | other | — | — | Use these options to select the iCloud features you want to use, manage iCloud storage, and set up iCloud+ features. | — |
| MD.02.c011 | Apple Account settings | Family | other | — | — | Use these options to set up and manage a Family Sharing group to share your subscriptions, purchases, location, and more, with up to five other family members. | — |
| MD.02.c012 | Apple Account settings | Media & Purchases | other | — | — | Use these options to update your account settings, subscriptions, and requirements for downloads and purchases. | — |
| MD.02.c013 | Apple Account settings | Sign in with Apple | other | — | — | See apps and websites you’ve signed in to using Sign in with Apple, adjust features, or stop using Sign in with Apple. | Appears only after you use Sign in with Apple to set up an account on an app or website. |
| MD.02.c014 | Apple Account settings | Devices | list | — | — | Use this list to review and manage the trusted devices that use your Apple Account. | — |
| MD.02.c015 | Apple Account settings | Contact Key Verification | other | — | — | Use this option to further verify that you’re exchanging messages only with the people you intend. | — |
| MD.02.c016 | Apple Account settings | Sign Out | other | — | — | Sign out of your Apple Account. | — |

### Apple Account > Media & Purchases: Manage Media & Purchases settings for your Apple Account on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl86b5abc5/27/mac/27) · topic `mchl86b5abc5` · path: System Settings > [your name] > Media & Purchases · "Open … for me" link: yes

Manage Apple Account purchase settings, subscriptions, and whether Touch ID or a password is required for downloads and purchases.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.02.c017 | Media & Purchases | Manage | button | — | — | Opens the Apple Account settings used for purchases from the App Store, iTunes Store, Apple Books, or Apple TV. | You must be signed in to your Apple Account in the App Store, iTunes Store, Apple Books, or Apple TV to access Media & Purchases settings. |
| MD.02.c018 | Media & Purchases | Manage | button | — | — | Opens settings for current subscriptions for Apple Music, Apple News, and more. | You must be signed in to your Apple Account in the App Store, iTunes Store, Apple Books, or Apple TV to access Media & Purchases settings. |
| MD.02.c019 | Media & Purchases | Use Touch ID for Purchases | toggle | On or off | — | When on, requires Touch ID to complete free downloads and purchases from the App Store, iTunes Store, Apple Books, and Apple TV. | If this is not turned on, you can choose password requirements instead.; Requires Touch ID to complete the specified downloads and purchases. |
| MD.02.c020 | Media & Purchases | Free Downloads | popup | Always Require; Never Require | — | Sets whether a password is required when downloading free content. | Available as a password option if Use Touch ID for Purchases is not turned on.; May require a password when downloading free content. |
| MD.02.c021 | Media & Purchases | Purchases and In-App Purchases | popup | Always Require; Never Require; “Require after 15 minutes” | — | Sets whether a password is required when purchasing from the App Store, iTunes Store, Apple Books, or Apple TV, or making a purchase from within an app. “Require after 15 minutes” requires a password only after 15 minutes have elapsed since the last purchase. | Available as a password option if Use Touch ID for Purchases is not turned on.; May require a password for purchases. |

Notes: The page does not specify a macOS version. You must be signed in to your Apple Account in the App Store, iTunes Store, Apple Books, or Apple TV to access Media & Purchases settings. If your name does not appear at the top of the sidebar, click “Sign in” and enter your Apple Account email address or phone number and password.

### Apple Account > Payment & Shipping: Change Payment & Shipping settings for your Apple Account on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlac6a3a51/27/mac/27) · topic `mchlac6a3a51` · path: Apple menu > System Settings > [your name] > Payment & Shipping · "Open … for me" link: no

Use Apple Account Payment & Shipping settings to view payment information and balances, and add or update the shipping information associated with your Apple Account.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.02.c022 | Payment & Shipping | Payment Method | info | — | — | Shows the payment method associated with your Apple Account, such as a credit card, debit card, or PayPal. The page says payment methods can be managed in iPhone or iPad Settings with iOS 12.2 or later, in the App Store on earlier iOS versions, or in the Mac App Store or iTunes Store (if available) on a Mac or PC. | The iPhone or iPad Settings route requires iOS 12.2 or later; the page also notes the Mac App Store or iTunes Store may not be available. |
| MD.02.c023 | Payment & Shipping | Apple Cash Balance | info | — | — | Shows your Apple Cash balance, which you can use with any Apple Pay–capable iPhone, iPad, or Apple Watch where you’re signed in to your Apple Account. | Use requires an Apple Pay–capable iPhone, iPad, or Apple Watch signed in to your Apple Account. |
| MD.02.c024 | Payment & Shipping | Apple Card Balance | info | — | — | Shows your current balance if you use Apple Card. | Shown if you use Apple Card. |
| MD.02.c025 | Shipping Address | Add a shipping address | button | — | — | Click to enter the name and shipping address to use for delivering purchases from the Apple Store. | After entering the name and address, click “Save”. |
| MD.02.c026 | Shipping Address | name and address | field | — | — | Enter the name and shipping address to use for delivering purchases from the Apple Store. | — |
| MD.02.c027 | Shipping Address | Save | button | — | — | Save the entered or updated name and shipping address. | — |
| MD.02.c028 | Shipping Address | Edit | button | — | — | Click to change the name or address. | After changing the name or address, click “Save”. |

Notes: If your name does not appear, click “Sign in,” enter your Apple Account email address or phone number, then enter your password. The supplied page text does not state a macOS version.

### Apple Account > Devices: Devices settings for your Apple Account on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl78de875c/27/mac/27) · topic `mchl78de875c` · path: Apple menu > System Settings > [your name] > Devices · "Open … for me" link: no

Review trusted devices that use your Apple Account and view device details, Find My status, coverage, Wallet and Apple Pay cards, or remove a device from the account.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.02.c029 | Devices | Device Info | info | — | — | See the specific model, software version, and serial number for a device. For mobile devices, you can also review the IMEI (International Mobile Equipment Identity), and the associated phone number, if applicable. | IMEI and associated phone number information applies to mobile devices. |
| MD.02.c030 | Devices | Find My [device] | info | — | — | Check whether Find My is on for the device. If it is, you can locate it in the Find My app or on iCloud.com. If not, see Set up Find My on all your devices in the iCloud User Guide. | — |
| MD.02.c031 | Devices | AppleCare & Warranty | info | — | — | Check coverage for the device. | — |
| MD.02.c032 | Devices | Wallet & Apple Pay | info | — | — | Check credit, debit, transit, and other cards used by Wallet and Apple Pay on the device. | — |
| MD.02.c033 | Wallet & Apple Pay | Remove Items | button | — | — | Remove all the cards used by Wallet and Apple Pay on the device. | Removes all the cards. |
| MD.02.c034 | Devices | Remove from Account | button | — | — | Delete the device from the list of trusted devices. | Doesn’t appear if you selected the device you’re currently using.; Deletes the device from the list of trusted devices. |

Notes: The page does not state a macOS version. To open these settings, choose Apple menu. If you don’t see your name, click “Sign in,” enter your Apple Account email address or phone number, then enter your password. The page lists “Add or remove trusted devices” and “Change your computer’s name or local hostname” under Popular topics.

### Apple Account > Personal Information: Change Personal Information settings for your Apple Account on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlfee1ae3a/27/mac/27) · topic `mchlfee1ae3a` · path: Apple menu > System Settings > [your name] > Personal Information · "Open … for me" link: no

Use Personal Information settings to change the name, birthday, age-range sharing options, and communication preferences associated with your Apple Account.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.02.c035 | Personal Information | Name | other | — | — | Change the name associated with the Apple Account. | — |
| MD.02.c036 | Personal Information | Birthday | other | — | — | Specify the birthday associated with your account. | — |
| MD.02.c037 | Personal Information | Age Range for Apps | other | — | — | Choose options for sharing your age range with apps that request this information. Your specific age and birthday aren’t shared with apps. | — |
| MD.02.c038 | Personal Information | Communication preferences | other | — | — | Choose whether to receive emails and communications about Apple products, services, software, and media, and Apple News newsletters. Announcements and newsletters are sent to the email address or addresses associated with your Apple Account. | This option doesn’t appear if the account is for a child under 13. Age restrictions vary by country or region. |

### Apple Account > Family Sharing: Set up Family Sharing on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mh3d6eb198/27/mac/27) · topic `mh3d6eb198` · path: System Settings > [your name] > Family · "Open … for me" link: yes

An adult organizer can invite up to five other family members to a Family Sharing group. Members can choose which features to use. The page also describes accepting an invitation and viewing recommended settings.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.02.c039 | Invite people to join your Family Sharing group | System Settings | other | — | — | Open the System Settings app. | — |
| MD.02.c040 | Invite people to join your Family Sharing group | your name | other | — | — | Click your name at the top of the sidebar. | If your name isn’t shown, sign in first. |
| MD.02.c041 | Invite people to join your Family Sharing group | Sign in | button | — | — | Sign in if your name isn’t shown. | If your name isn’t shown.; Requires an Apple Account email address or phone number and password. |
| MD.02.c042 | Invite people to join your Family Sharing group | email address or phone number | field | — | — | Enter your Apple Account email address or phone number when signing in. | If you need to sign in. |
| MD.02.c043 | Invite people to join your Family Sharing group | password | field | — | — | Enter your password when signing in. | If you need to sign in.; Authentication. |
| MD.02.c044 | Invite people to join your Family Sharing group | Open Apple Account settings for me | link | — | — | Opens Apple Account settings. | — |
| MD.02.c045 | Invite people to join your Family Sharing group | Family | other | — | — | Click Family. | — |
| MD.02.c046 | Invite people to join your Family Sharing group | Open Family settings for me | link | — | — | Opens Family settings. | — |
| MD.02.c047 | Invite people to join your Family Sharing group | Set up Family | button | — | — | Click to begin setting up Family Sharing. | — |
| MD.02.c048 | Invite people to join your Family Sharing group | Invite People | button | — | — | Click after Set up Family. | — |
| MD.02.c049 | Invite people to join your Family Sharing group | Invite People | button | — | — | Click to choose an invitation method. | — |
| MD.02.c050 | Invite people to join your Family Sharing group | Invite in Person | radio | Invite in Person; AirDrop; Mail; Messages | — | Choose this to invite someone who is currently with you. | The other person must be with you.; The other person enters their Apple Account email address or phone number and password on your Mac. |
| MD.02.c051 | Invite people to join your Family Sharing group | Continue | button | — | — | Click after choosing an invitation method. | — |
| MD.02.c052 | Invite people to join your Family Sharing group | Apple Account email address or phone number | field | — | — | The invited person enters this on your Mac for an in-person invitation. | For an in-person invitation. |
| MD.02.c053 | Invite people to join your Family Sharing group | password | field | — | — | The invited person enters their password on your Mac for an in-person invitation. | For an in-person invitation.; Authentication. |
| MD.02.c054 | Accept an invitation to a Family Sharing group | Allow | button | — | — | Click on your device to see a code when invited in person. | For an in-person invitation.; Displays a code for the organizer to enter on their device. |
| MD.02.c055 | Accept an invitation to a Family Sharing group | code | info | — | — | The organizer enters the code shown on your device. | For an in-person invitation. |
| MD.02.c056 | Accept an invitation to a Family Sharing group | AirDrop, Mail, or Messages | other | AirDrop; Mail; Messages | — | Use the invitation received through one of these methods, then click Join Family. | If the organizer sent an invitation. |
| MD.02.c057 | Accept an invitation to a Family Sharing group | Join Family | button | — | — | Click to accept an invitation received through AirDrop, Mail, or Messages. | If the organizer sent an invitation. |
| MD.02.c058 | Accept an invitation to a Family Sharing group | Apple menu | other | — | — | The page begins an instruction to choose Apple menu to accept an invitation in System Settings, but the instruction is incomplete. | — |
| MD.02.c059 | View recommended settings | System Settings | other | — | — | Open the System Settings app. | — |
| MD.02.c060 | View recommended settings | your name | other | — | — | Click your name at the top of the sidebar. | If your name isn’t shown, sign in first. |
| MD.02.c061 | View recommended settings | Sign in | button | — | — | Sign in if your name isn’t shown. | If your name isn’t shown.; Requires an Apple Account email address or phone number and password. |
| MD.02.c062 | View recommended settings | Apple Account email address or phone number | field | — | — | Enter your Apple Account email address or phone number when signing in. | If you need to sign in. |
| MD.02.c063 | View recommended settings | password | field | — | — | Enter your password when signing in. | If you need to sign in.; Authentication. |
| MD.02.c064 | View recommended settings | Open Apple Account settings for me | link | — | — | Opens Apple Account settings. | — |
| MD.02.c065 | View recommended settings | Family | other | — | — | Click Family. | — |
| MD.02.c066 | View recommended settings | Recommended for Your Family | other | — | — | Click to view suggestions for getting the most out of Family Sharing. | — |

Notes: The page does not state a macOS version. Family Sharing supports the organizer and up to five other family members. Children under 13 can’t create an Apple Account on their own. A family organizer, parent, or guardian can create one for a child. A Child Account is required for children under 13 and is available for children up to 18; age varies by country or region. To join, you must be signed in to the Apple Account the organizer invited. If you don’t have an Apple Account, you must create one before joining. The in-person invitation and invitation-sending methods are described as alternatives; no default is stated. The instruction for accepting an invitation in System Settings ends after “On your Mac, choose Apple menu” and does not provide the rest of the path.

### Apple Account > Sign-In & Security: Set up your Apple Account sign-in and security information on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlc7ba8f3e/27/mac/27) · topic `mchlc7ba8f3e` · path: System Settings > your name at the top of the sidebar > Sign-In & Security · "Open … for me" link: yes

Manage the email addresses, phone numbers, password, two-factor authentication, recovery options, Legacy Contact, and Automatic Verification associated with your Apple Account.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.02.c067 | System Settings | Sign in | button | — | — | If your name isn’t at the top of the sidebar, sign in with your Apple Account email address or phone number and password. | Appears as the alternative when your name isn’t shown in the sidebar. |
| MD.02.c068 | Sign-In & Security | Email & Phone Numbers | link | — | — | Manage the email addresses and phone numbers you can use to sign in to your Apple Account. | — |
| MD.02.c069 | Email & Phone Numbers | Add an Email or Phone Number | button | Email address or phone number | — | Add an email address or phone number to use to sign in to your Apple Account. | — |
| MD.02.c070 | Email & Phone Numbers | Remove from Account | button | — | — | Remove the selected email address or phone number from your account. | Select an email address or phone number first. |
| MD.02.c071 | Sign-In & Security | Password | link | — | — | Change the password that protects information associated with your Apple Account, including payment and shipping information, iCloud information, and media and purchases information. | — |
| MD.02.c072 | Password | Change Password | button | — | — | Change your Apple Account password. | — |
| MD.02.c073 | Sign-In & Security | Two-Factor Authentication | link | On or off | Turned on automatically for most Mac computers. | View whether two-factor authentication is turned on. When it’s on, you can add or remove trusted phone numbers, set up security keys, or get a verification code. | — |
| MD.02.c074 | Two-Factor Authentication | Turn on Two-Factor Authentication | button | — | — | Begin using two-factor authentication. | Available when two-factor authentication is off.; After you set up two-factor authentication, you must use it when you sign in to your account; you can’t turn it off. |
| MD.02.c075 | Sign-In & Security | Recovery Contacts | link | Up to five people | — | Choose people you know and trust to help verify your identity and help you regain access if you’re locked out. Recovery contacts can’t access your account or data. | Appears only when you’re using two-factor authentication. |
| MD.02.c076 | Recovery Contacts | Add Recovery Contact | button | — | — | Add a person as an account recovery contact and follow the onscreen instructions. | Recovery Contacts appears only when you’re using two-factor authentication. |
| MD.02.c077 | Recovery Contacts | Get Recovery Code | button | — | — | Get a recovery code to help a friend or family member regain access to their account. | Select the friend or family member’s name first. |
| MD.02.c078 | Sign-In & Security | Recovery Key | link | — | — | Set up a recovery key to increase the security of your account. | — |
| MD.02.c079 | Recovery Key | Turn On | button | — | — | Turn on Recovery Key and follow the onscreen instructions. | — |
| MD.02.c080 | Sign-In & Security | Legacy Contact | link | — | — | Designate people who can access your Apple Account data in the event of your death. A Legacy Contact can be anyone you choose; they don’t need an Apple Account or an Apple device. | — |
| MD.02.c081 | Legacy Contact | Add Legacy Contact | button | — | — | Designate a Legacy Contact and follow the onscreen instructions. | — |
| MD.02.c082 | Sign-In & Security | Automatic Verification | toggle | On or off | — | When turned on, allows iCloud to automatically and privately verify your device and account to bypass CAPTCHAs in apps and on the web. | — |

Notes: The page does not state a macOS version. The page says two-factor authentication is turned on automatically for most Mac computers.

### Apple Account > iCloud: Change iCloud settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mh36817/27/mac/27) · topic `mh36817` · path: Apple menu > System Settings > [your name] > iCloud · "Open … for me" link: no

Use Apple Account iCloud settings to turn iCloud and iCloud+ features on or off and manage iCloud storage.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.02.c083 | iCloud Storage bar | Manage | button | Change your iCloud storage plan; share iCloud+ with your family; remove an iPhone or iPad backup; remove photos, videos, and data from iCloud. | — | Opens iCloud storage management. | — |
| MD.02.c084 | Saved to iCloud | Photos | other | — | — | Click an app to see more details about its syncing status. | — |
| MD.02.c085 | Saved to iCloud | Drive | other | — | — | Click an app to see more details about its syncing status. | — |
| MD.02.c086 | Saved to iCloud | Passwords | other | — | — | Click an app to see more details about its syncing status. | — |
| MD.02.c087 | Saved to iCloud | Notes | other | — | — | Click an app to see more details about its syncing status. | — |
| MD.02.c088 | Saved to iCloud | Messages | other | — | — | Click an app to see more details about its syncing status. | — |
| MD.02.c089 | Saved to iCloud | Mail | other | — | — | Click an app to see more details about its syncing status. | — |
| MD.02.c090 | Saved to iCloud | See All | button | — | — | Opens more apps to manage. | — |
| MD.02.c091 | Photos | Sync this Mac | other | On or off. | — | Stores photos and videos using iCloud Photos and makes them accessible in Photos on devices signed in to the Apple Account. Also supports Shared Albums and iCloud Shared Photo Library. | — |
| MD.02.c092 | Drive | Sync this Mac | other | On or off. | — | Stores files in iCloud Drive and makes them accessible on devices signed in to the Apple Account. | — |
| MD.02.c093 | Drive | Desktop and Documents folders | other | — | — | Choose whether to store the Desktop and Documents folders in iCloud Drive. | — |
| MD.02.c094 | Drive | older documents in iCloud | other | — | — | Choose whether to store older documents in iCloud when disk space is needed. | — |
| MD.02.c095 | Drive | Which apps store data in iCloud Drive | other | — | — | Choose which apps store data in iCloud Drive. | — |
| MD.02.c096 | Passwords | Sync this Mac | other | On or off. | — | Securely stores information such as passwords and credit card numbers in iCloud and makes it accessible on devices signed in to the Apple Account. | — |
| MD.02.c097 | Notes | Sync this Mac | other | On or off. | — | Stores notes in iCloud and makes them accessible on devices signed in to the Apple Account. | — |
| MD.02.c098 | Messages | Use on this Mac | other | On or off. | — | Stores messages in iCloud and makes them accessible on devices signed in to the Apple Account. The settings also show how many messages are in iCloud, let you choose how long to keep messages, and let you sync your Mac. | — |
| MD.02.c099 | Messages | how long to keep messages | other | — | — | Choose how long to keep messages. | — |
| MD.02.c100 | Messages | sync your Mac | other | — | — | Sync your Mac. | — |
| MD.02.c101 | Mail | Sync this Mac | other | On or off. | — | Stores iCloud Mail email in iCloud and makes it accessible in Mail on devices signed in to the Apple Account. | If you don’t already have an iCloud email address, you’re prompted to create one. |
| MD.02.c102 | Find My Mac | Turn On | button | — | — | Turns on Find My to locate, lock, or remotely erase devices signed in to the Apple Account. | Location Services must be on to use Find My Mac. |
| MD.02.c103 | Contacts | Contacts | other | — | — | Stores contacts in iCloud so they can be accessed on devices signed in to the Apple Account. | — |
| MD.02.c104 | iCloud Calendar | iCloud Calendar | other | — | — | Stores calendar information in iCloud so it can be accessed on devices signed in to the Apple Account. | — |
| MD.02.c105 | Reminders | Reminders | other | — | — | Stores reminders in iCloud so they can be accessed on devices signed in to the Apple Account. | — |
| MD.02.c106 | Safari | Safari | other | — | — | Stores open tabs in iCloud Tabs, Reading List, and bookmarks so they can be accessed on devices signed in to the Apple Account. | — |
| MD.02.c107 | News | News | other | — | — | Stores followed channels and topics, saved stories, and reading history in iCloud for access on devices signed in to the Apple Account. | — |
| MD.02.c108 | Stocks | Stocks | other | — | — | Stores stock watchlist information in iCloud for viewing on devices signed in to the Apple Account. | — |
| MD.02.c109 | Home | Home | other | — | — | Allows internet communication through iCloud using the Home app to control internet-connected home devices on Apple devices signed in to the Apple Account. | — |
| MD.02.c110 | Wallet | Wallet | other | — | — | Stores cards in iCloud for use on devices signed in to the Apple Account. | — |
| MD.02.c111 | Siri | Siri | other | — | — | Stores Siri settings in iCloud for use on devices signed in to the Apple Account. | — |
| MD.02.c112 | Image Playground | Image Playground | other | — | — | Stores images and characters from Image Playground in iCloud for use on devices signed in to the Apple Account. | Image Playground is available with Apple Intelligence. Apple Intelligence on macOS 27 is not available in all languages or regions. |
| MD.02.c113 | Freeform | Freeform | other | — | — | Stores Freeform boards in iCloud for use on devices signed in to the Apple Account. | — |
| MD.02.c114 | Journal | Journal | other | — | — | Stores journal entries in iCloud for use on devices signed in to the Apple Account. | — |
| MD.02.c115 | FaceTime | FaceTime | other | — | — | Stores call history and video messages in iCloud for access on devices signed in to the Apple Account. | — |
| MD.02.c116 | Get More with iCloud+ | Upgrade to iCloud+ | button | View available plans. | — | Begins the iCloud+ subscription process. | Shown if you don’t subscribe to iCloud+. |
| MD.02.c117 | Manage Plan | Select an option | other | Available storage plan options; exact options are not listed. | — | Select a storage option when upgrading, then click Next. | Applies if you subscribe to iCloud+.; An upgrade provides more storage per month. |
| MD.02.c118 | Manage Plan | Next | button | — | — | Continues after selecting an upgrade option. | Applies if you subscribe to iCloud+. |
| MD.02.c119 | Manage Plan | Downgrade Options | button | Downgrade storage options; exact options are not listed. | — | Opens options to downgrade the amount of storage. | Applies if you subscribe to iCloud+.; A downgrade takes effect after the current monthly or annual subscription ends. Downgrading to 5 GB removes access to iCloud+ features. |
| MD.02.c120 | Family | Set up Family Sharing | link | — | — | Set up Family Sharing to share an iCloud+ subscription and more. | — |
| MD.02.c121 | Apple Invites | Apple Invites | info | — | — | The page says an iCloud+ subscription can be used to host an event. | Requires an iCloud+ subscription. |
| MD.02.c122 | Private Relay | Private Relay | info | — | — | Use iCloud Private Relay to hide the IP address and Safari browsing activity and protect unencrypted internet traffic. | Requires an iCloud+ subscription. |
| MD.02.c123 | Hide My Email | Hide My Email | info | — | — | Generate unique, random email addresses that forward to a personal email account; as many addresses as needed can be created. | Requires an iCloud+ subscription. |
| MD.02.c124 | Custom Email | Custom Email | info | — | — | Send and receive email from a personalized email address with iCloud Mail. | Requires an iCloud+ subscription. |
| MD.02.c125 | Advanced Data Protection | Advanced Data Protection | info | — | — | Uses end-to-end encryption to protect the majority of iCloud data. | Not available in all countries or regions. A recovery contact or recovery key must be set up on the account.; Apple won’t have the keys required to recover the data. |
| MD.02.c126 | iCloud.com | Allow Data Access | other | On or off. | — | Controls access to mail, contacts, calendar, photos, notes, reminders, files, and documents on iCloud.com and privacy.apple.com. | — |
| MD.02.c127 | iCloud.com | Allow Search | other | On or off. | — | Allows trusted devices to provide local search results to iCloud Photos and iCloud Drive on iCloud.com. | — |
| MD.02.c128 | iCloud.com | Advanced | button | — | — | Opens options to export or delete a report of search queries from the last 28 days on iCloud.com. | — |

Notes: The supplied page text ends the navigation instruction at “choose Apple menu”; the rest of the navigation path is not included in the supplied text. No defaults are stated. The Saved to iCloud circle indicates storage status: available space means apps are syncing and new items save to iCloud; almost-full storage may cause some apps to fail to sync; full storage means new items don’t save; a further status indicates syncing or backups aren’t enabled, or there is no internet connection. iCloud requires an internet connection; some iCloud features have minimum system requirements. iCloud and iCloud+ may not be available in all areas, and iCloud+ features may vary by area. Apple Intelligence availability depends on language, region, and device; usage limits may apply.

## 13.3 MD.03 Family

Status: **PARTIAL**. Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13).


### Family: Change Family settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl37e74c6a/27/mac/27) · topic `mchl37e74c6a` · path: Apple menu > System Settings > Family · "Open … for me" link: yes

Use Family settings to view and manage Family Sharing members, add family members, and manage shared subscriptions, purchases, and location.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.03.c001 | Family | Family members list | list | — | — | Click a family member’s name to see details of that person’s settings. For Child Account settings and parental controls, see Apple Devices for Kids and Families. | — |
| MD.03.c002 | Family | Add Member | button | — | — | Add another family member to your Family Sharing group. | — |
| MD.03.c003 | Family | Recommended For Your Family | other | — | — | Set up recommended settings to get the most out of Family Sharing. | — |
| MD.03.c004 | Family | Subscriptions | other | — | — | View which of your subscribed services are shared with your family and which subscriptions family members share with you. Start or stop sharing eligible subscriptions. | Sharing applies to eligible subscriptions. |
| MD.03.c005 | Family | Purchase Sharing | other | — | — | Set up and manage purchases shared with other family members, and see who’s sharing purchases. | — |
| MD.03.c006 | Family | Location Sharing | other | — | — | Share your location and the location of your devices with other family members. | — |

Notes: If you don’t see Family, set up Family Sharing. For setting up Screen Time or parental control features for a child, the page refers to Apple Devices for Kids and Families. The page also refers to Apple Support information about bundling Apple subscriptions with Apple One.

## 13.4 MD.04 Wi-Fi

Status: **PARTIAL**. Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13).


### Wi-Fi: Use Wi-Fi settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl339ee00e/27/mac/27) · topic `mchl339ee00e` · path: System Settings > Wi-Fi · "Open … for me" link: yes

Use Wi-Fi settings to turn Wi-Fi on or off, connect to Wi-Fi networks and Personal Hotspots, choose automatic-connection behavior, and access additional Wi-Fi settings.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.04.c001 | Turn Wi-Fi on or off | Wi-Fi | toggle | On or off | — | Turn Wi-Fi on or off. | — |
| MD.04.c002 | Connect to a Wi-Fi network or Personal Hotspot | Wi-Fi networks and Personal Hotspots | list | — | — | Move the pointer over the name of a network or Personal Hotspot to join it. | — |
| MD.04.c003 | Connect to a Wi-Fi network or Personal Hotspot | Connect | button | — | — | Connect to the selected Wi-Fi network or Personal Hotspot. | — |
| MD.04.c004 | Connect to a Wi-Fi network or Personal Hotspot | Other | button | — | — | Use this when the network you want to join isn’t shown; enter its network name and password, then click OK. | — |
| MD.04.c005 | Connect to a Wi-Fi network or Personal Hotspot | network name | field | — | — | Enter the name of a network that isn’t shown. | — |
| MD.04.c006 | Connect to a Wi-Fi network or Personal Hotspot | password | field | — | — | Enter the password for a network that isn’t shown. | — |
| MD.04.c007 | Connect to a Wi-Fi network or Personal Hotspot | OK | button | — | — | Confirm the entered network name and password. | — |
| MD.04.c008 | Choose whether to connect to known networks automatically | “Ask to join networks” | popup | Off, Notify, Ask | — | Choose whether to connect to known networks automatically. | — |
| MD.04.c009 | Choose whether to automatically discover nearby Personal Hotspots | “Ask to join hotspots” | popup | Never, Ask To Join, Automatic | — | Choose whether to automatically discover nearby Personal Hotspots. | — |
| MD.04.c010 | Enter or change Wi-Fi settings | Details | button | — | — | Open additional Wi-Fi settings. | — |

Notes: The page suggests using the Wi-Fi status menu in the menu bar to quickly turn Wi-Fi on or off or connect to a Wi-Fi network. The page does not state a macOS version, defaults, hardware or account requirements, or restart/authentication effects. The page directs users to another Apple Support article if they can’t see or connect to a Personal Hotspot; that article is not included here.

## 13.5 MD.05 Bluetooth

Status: **PARTIAL**. Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13).


### Bluetooth: Bluetooth settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/blth8111/27/mac/27) · topic `blth8111` · path: Apple menu > System Settings > Bluetooth · "Open … for me" link: no

Use Bluetooth settings to turn Bluetooth on or off, manage previously connected devices, and connect to nearby wireless devices.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.05.c001 | Bluetooth settings | Turn Bluetooth on or off | toggle | — | — | Turns Bluetooth on or off on your Mac. | — |
| MD.05.c002 | Bluetooth settings | My Devices | list | — | — | Manage devices previously connected to your Mac that can automatically reconnect. | — |
| MD.05.c003 | Bluetooth settings | Nearby Devices | list | — | — | Connect a new wireless device to your Mac and see other discoverable wireless devices in the area. | — |

### Headphones / AirPods: Use AirPods with your Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchld2f124d1/27/mac/27) · topic `mchld2f124d1` · path: System Settings > Bluetooth; System Settings > [name of your AirPods] in the sidebar · "Open … for me" link: yes

Pair AirPods with your Mac in Bluetooth settings, then use the AirPods pane in System Settings to change model-specific controls.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.05.c004 | Pair your AirPods with your Mac | Bluetooth | link | — | — | Opens Bluetooth settings in System Settings. | — |
| MD.05.c005 | Pair your AirPods with your Mac | Bluetooth status menu | other | — | — | The page says you can add this to make it easier to connect your AirPods. | — |
| MD.05.c006 | Pair your AirPods with your Mac | Connect | button | — | — | Connects to your AirPods from the list of devices. | AirPods must be in the list of devices. |
| MD.05.c007 | Pair your AirPods with your Mac | when your AirPods connect to your Mac | other | automatically; only when last connected to your Mac | — | The page says you can change whether AirPods connect automatically or only when last connected to your Mac. | — |
| MD.05.c008 | Change AirPods settings on your Mac | the name of your AirPods | link | — | — | Select your AirPods in the sidebar to open their settings. | Wear your AirPods and make sure they’re connected to your Mac. You may need to scroll down. |

Notes: Options vary by model. The page does not specify individual AirPods controls or a macOS version.

## 13.6 MD.06 Network

Status: **PARTIAL**. Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13).


### Network: Change network service settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp1102/27/mac/27) · topic `mchlp1102` · path: Apple menu > Network settings · "Open … for me" link: yes

Use Network settings to change a network service’s settings. Enter settings provided by your ISP or network administrator.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.06.c001 | Step 1 | Open Network settings for me | link | — | — | Opens Network settings. | — |
| MD.06.c002 | Step 2 | service | list | Ethernet; Wi-Fi (examples) | — | Click a network service to select it. | If the expected service isn’t shown, see Set up a network service. |
| MD.06.c003 | Step 3 | Details | button | — | — | Opens the selected service’s settings. | — |
| MD.06.c004 | Step 3 | OK | button | — | — | Confirms the changed settings. | — |

Notes: The page does not name the individual settings available under Details. Use settings received from your ISP or network administrator.

### Network > Thunderbolt Bridge: Use IP over Thunderbolt to connect Mac computers

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchld53dd2f5/27/mac/27) · topic `mchld53dd2f5` · path: Apple menu > Network settings > Thunderbolt Bridge > Details · "Open … for me" link: yes

Connect two Thunderbolt-equipped Mac computers with a Thunderbolt cable. By default, they receive IP addresses using DHCP; if needed, configure IPv4, DNS, or search domain addresses manually.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.06.c005 | — | Network settings for me | link | — | — | Opens Network settings. | — |
| MD.06.c006 | — | Thunderbolt Bridge | list | — | — | Select the Thunderbolt Bridge service. | If the service is not shown, add one. |
| MD.06.c007 | — | Set up a network service | link | — | — | Reference link for adding a network service. | Use if Thunderbolt Bridge is not shown. |
| MD.06.c008 | — | Details | button | — | — | Opens details for the Thunderbolt Bridge service. | — |
| MD.06.c009 | — | TCP/IP | other | — | — | Opens TCP/IP settings. | — |
| MD.06.c010 | — | Configure IPv4 | popup | Options are not listed. | — | Choose an IPv4 configuration option. | Use to enter an IPv4 address if needed. |
| MD.06.c011 | — | IP address | field | — | — | Enter the IPv4 address. | If you need to enter an IP address manually. |
| MD.06.c012 | — | subnet mask | field | — | — | Enter a subnet mask. | If needed. |
| MD.06.c013 | — | router address | field | — | — | Enter a router address. | If needed. |
| MD.06.c014 | — | DNS | other | — | — | Opens DNS settings. | Use if you need to enter DNS or search domain addresses. |
| MD.06.c015 | — | DNS or search domain addresses | field | — | — | Enter the addresses. | If you need to enter DNS or search domain addresses. |
| MD.06.c016 | — | OK | button | — | — | Confirm the settings. | — |

Notes: The page does not state a macOS version. By default, the connected computers receive IP addresses using DHCP. The page does not list the options in the Configure IPv4 pop-up menu. The Finder file-transfer instructions say the other Mac must have file sharing turned on if it does not appear in Network.

### Network > Ethernet: Ethernet settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mh11939/27/mac/27) · topic `mh11939` · path: Apple menu > System Settings > Network > Ethernet · "Open … for me" link: no

Use Ethernet network settings to set up and manage an Ethernet connection.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.06.c017 | Ethernet settings | *service name* | list | — | — | The name of the network service and its connection status indicator. If connected, the service’s current settings are listed below its name. | Current settings are listed below the name if the service is connected. |
| MD.06.c018 | Ethernet settings | Details | button | — | — | Change settings for the network service. | — |
| MD.06.c019 | Ethernet settings | Delete Service | button | — | — | Remove the network service. | — |
| MD.06.c020 | Ethernet settings | Make Inactive | button | — | — | Deactivate the network service. | — |
| MD.06.c021 | Ethernet Details settings | Name | field | — | — | The name of the network service. To change it, select the name and enter a new name. | — |
| MD.06.c022 | Ethernet Details settings | Limit IP address tracking | checkbox | — | — | Hide your IP address from known trackers in Mail and Safari. | If you subscribe to iCloud+ and have Private Relay turned on in iCloud settings, that condition applies when this option is deselected.; Deselecting this option turns off Private Relay if you subscribe to iCloud+ and have it turned on in iCloud settings. |
| MD.06.c023 | Ethernet Details settings | TCP/IP | other | Configure IPv4; configure IPv6; renew the DHCP lease. | — | Configure IPv4 and IPv6, or renew the DHCP lease. | — |
| MD.06.c024 | Ethernet Details settings | DNS | other | DNS servers; search domains. | — | Configure DNS servers and search domains. | — |
| MD.06.c025 | Ethernet Details settings | WINS | other | — | — | Configure Windows Internet Name Service (WINS) settings. | — |
| MD.06.c026 | Ethernet Details settings | 802.1X | other | Connect to a network protected by 802.1X; view 802.1X profile settings; enable automatic connection. | — | Connect to a network protected by 802.1X, view 802.1X profile settings, and enable automatic connection. | Connecting requires a network protected by 802.1X. |
| MD.06.c027 | Ethernet Details settings | Proxies | other | — | — | Configure internet proxies. | — |
| MD.06.c028 | Ethernet Details settings | Hardware | other | — | — | Configure hardware settings and view the hardware MAC address. | — |

Notes: The page does not specify a macOS version. The page lists related topics: “Connect your Mac to the internet using Ethernet” and “Change the order of the network services your Mac uses.” See also: Apple Support article “Manage iCloud Private Relay for specific websites, networks, or system settings.”

### Network > Firewall: Change Firewall settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mh11783/27/mac/27) · topic `mh11783` · path: Apple menu > System Settings > Network > Firewall · "Open … for me" link: no

Use Firewall settings to turn on the firewall and control incoming connections to your Mac.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.06.c029 | Firewall settings | Firewall | toggle | — | — | Prevent unwanted connections from the internet or other networks. | — |
| MD.06.c030 | Firewall settings | Options | button | — | — | Choose how much to block incoming connections. | — |
| MD.06.c031 | Options | Block all incoming connections | checkbox | — | — | Prevent incoming connections to nonessential services and apps. Basic internet services are a set of apps that allow your Mac to find services provided by other computers on the network. This setting prevents connections to all other sharing services. | — |
| MD.06.c032 | Options | Add | button | — | — | Add an app or service to allow incoming connections. | — |
| MD.06.c033 | Options | Remove | button | — | — | Remove an app or service from allowing incoming connections. | — |
| MD.06.c034 | Options | Automatically allow built-in software to receive incoming connections | checkbox | — | — | Allow built-in apps and services that are signed by a valid certificate authority to be automatically added to the list of allowed apps, without your authorization. | — |
| MD.06.c035 | Options | Automatically allow downloaded signed software to receive incoming connections | checkbox | — | — | Allow downloaded apps and services that are signed by a valid certificate authority to be automatically added to the list of allowed apps, without your authorization. | — |
| MD.06.c036 | Options | Enable stealth mode | checkbox | — | — | Prevent your Mac from responding to probing requests that can be used to reveal its existence. The Mac still answers requests from authorized apps, but unauthorized requests such as ICMP (ping) get no response. | — |

Notes: The supplied page text does not state a macOS version. Certain shared services can connect through the firewall when they are turned on in Sharing settings. To prevent connections to those incoming services, turn off the service in Sharing settings. See also: “Block connections to your Mac with a firewall” and “Use stealth mode to keep your Mac more secure.”

### Network > Proxies: Change proxy settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp2591/27/mac/27) · topic `mchlp2591` · path: — · "Open … for me" link: no

Use proxy settings to manage internet proxy services and enter proxy server settings.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.06.c037 | — | Auto proxy discovery | checkbox | — | — | Configure proxy server settings automatically. | — |
| MD.06.c038 | — | Automatic proxy configuration | checkbox | — | — | Use a proxy auto-configuration (PAC) file. | — |
| MD.06.c039 | — | URL | field | — | — | Enter the address of the PAC file. | — |
| MD.06.c040 | — | Web proxy (HTTP) | checkbox | — | — | Use a web proxy server. | — |
| MD.06.c041 | — | server address | field | — | — | Enter the server address. | — |
| MD.06.c042 | — | port number | field | — | — | Enter the port number. | — |
| MD.06.c043 | — | Proxy server requires password | checkbox | — | — | Turn on if needed. | If needed. |
| MD.06.c044 | — | user name | field | — | — | Enter your user name. | If needed, after turning on “Proxy server requires password.” |
| MD.06.c045 | — | password | field | — | — | Enter your password. | If needed, after turning on “Proxy server requires password.” |
| MD.06.c046 | — | Secure web proxy (HTTPS) | checkbox | — | — | Use a secure web proxy server. | — |
| MD.06.c047 | — | server address | field | — | — | Enter the server address. | — |
| MD.06.c048 | — | port number | field | — | — | Enter the port number. | — |
| MD.06.c049 | — | Proxy server requires password | checkbox | — | — | Turn on if needed. | If needed. |
| MD.06.c050 | — | user name | field | — | — | Enter your user name. | If needed, after turning on “Proxy server requires password.” |
| MD.06.c051 | — | password | field | — | — | Enter your password. | If needed, after turning on “Proxy server requires password.” |
| MD.06.c052 | — | SOCKS proxy | checkbox | — | — | Use a SOCKS proxy server. | — |
| MD.06.c053 | — | server address | field | — | — | Enter the server address. | — |
| MD.06.c054 | — | port number | field | — | — | Enter the port number. | — |
| MD.06.c055 | — | Proxy server requires password | checkbox | — | — | Turn on if needed. | If needed. |
| MD.06.c056 | — | user name | field | — | — | Enter your user name. | If needed, after turning on “Proxy server requires password.” |
| MD.06.c057 | — | password | field | — | — | Enter your password. | If needed, after turning on “Proxy server requires password.” |
| MD.06.c058 | — | Exclude simple hostnames | checkbox | — | — | Bypass proxies for network destinations with simple hostnames (for example, https://server). | — |
| MD.06.c059 | — | Bypass proxy settings for these hosts & domains | field | — | — | Enter the hosts and domains to bypass. Separate multiple items with commas. | — |

Notes: See also: Apple Support article “TCP and UDP ports used by Apple software products.” The page does not state a macOS version or navigation path.

### Network > Network Details advanced Wi-Fi: Wi-Fi settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mh11935/27/mac/27) · topic `mh11935` · path: Apple menu > System Settings > Wi-Fi · "Open … for me" link: no

Use Wi-Fi settings to set up and manage your Wi-Fi connection, connect to networks and hotspots, and change network options.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.06.c060 | Wi-Fi settings | Wi-Fi | toggle | On; Off | — | Turn Wi-Fi on or off. | — |
| MD.06.c061 | Wi-Fi settings | Wi-Fi network name | info | — | — | Shows the name of the connected network and the connection status indicator. | — |
| MD.06.c062 | Wi-Fi settings | Details | button | — | — | View IP and router addresses, and change settings for the network. | — |
| MD.06.c063 | Wi-Fi settings | Personal Hotspots | list | — | — | Shows available Personal Hotspots. To connect, move the pointer over the hotspot and click Connect. | — |
| MD.06.c064 | Wi-Fi settings | Known Networks | list | — | — | Shows available networks you’ve connected to previously. Move the pointer over a network and click Connect to join it. | — |
| MD.06.c065 | Wi-Fi settings > Known Networks | Auto-Join | other | On; Off | — | Quickly toggle automatic joining for a known Wi-Fi network. | — |
| MD.06.c066 | Wi-Fi settings > Known Networks | Copy Password | other | — | — | Copy the password of a known Wi-Fi network. | — |
| MD.06.c067 | Wi-Fi settings > Known Networks | Network Settings | other | — | — | Open settings for a known Wi-Fi network. | — |
| MD.06.c068 | Wi-Fi settings > Known Networks | Forget This Network | other | — | — | Remove a known Wi-Fi network from the list. | Removes the network from the list. |
| MD.06.c069 | Wi-Fi settings | Other Networks | list | — | — | Shows available nearby networks you haven’t connected to previously. To join one, move the pointer over it and click Connect, then enter its password. | Connecting to a password-protected network requires entering its password. |
| MD.06.c070 | Wi-Fi settings > Other Networks | Other | button | — | — | Connect to a network that isn’t listed by entering its network name, security type, and password. | — |
| MD.06.c071 | Wi-Fi settings | Ask to join networks | popup | Off; Notify; Ask | — | Choose whether to receive a notification when networks you haven’t joined previously are available. | — |
| MD.06.c072 | Wi-Fi settings | Ask to join hotspots | popup | Never; Ask To Join; Automatic | — | Choose whether to automatically join an available Personal Hotspot when no known Wi-Fi networks are found. | — |
| MD.06.c073 | Wi-Fi settings | Advanced | button | — | — | Set or view additional Wi-Fi information, including known networks, administrator authorization requirements, and the Wi-Fi MAC address. | — |
| MD.06.c074 | Details button settings | Automatically join this network | toggle | On; Off | — | Automatically connect to the specified network when it’s available. | — |
| MD.06.c075 | Details button settings | Private Wi-Fi address | popup | Off; Fixed; Rotating | — | Choose how the Mac uses a private Wi-Fi address to help reduce tracking by Wi-Fi network operators. Off allows identification and tracking on this network and across other networks. Fixed allows tracking on this network but helps reduce tracking across other networks. Rotating reduces tracking on this network and across other networks. | — |
| MD.06.c076 | Details button settings | Wi-Fi address | info | — | — | Shows the current Wi-Fi address. | — |
| MD.06.c077 | Details button settings | Low data mode | toggle | On; Off | — | Reduce data usage over the Wi-Fi network. | — |
| MD.06.c078 | Details button settings | Wi-Fi 6E mode | popup | Automatic; Off | Automatic | Choose a Wi-Fi 6E mode for the network. Automatic uses the network’s 2.4 GHz, 5 GHz, and 6 GHz wireless bands. Off does not use the network’s 6 GHz band. | Off is available only when using a computer that supports Wi-Fi 6E and the Wi-Fi 6E network uses the same network name for all bands. |
| MD.06.c079 | Details button settings | Limit IP address tracking | toggle | On; Off | — | Hide your IP address from known trackers in Mail and Safari. | If you subscribe to iCloud+ and have Private Relay turned on in iCloud settings, deselecting this option turns Private Relay off. |
| MD.06.c080 | Details button settings | IP address | info | — | — | Shows the IP address for the Wi-Fi connection. | — |
| MD.06.c081 | Details button settings | Router | info | — | — | Shows the router address for the Wi-Fi connection. | — |
| MD.06.c082 | Details button settings | Forget This Network | button | — | — | Remove the network. | Removes the network. |
| MD.06.c083 | Details button settings | TCP/IP | other | — | — | Configure IPv4 and IPv6, or renew the DHCP lease. | — |
| MD.06.c084 | Details button settings | DNS | other | — | — | Configure DNS servers and search domains. | — |
| MD.06.c085 | Details button settings | WINS | other | — | — | Configure Windows Internet Name Service (WINS) settings. | — |
| MD.06.c086 | Details button settings | 802.1X | other | — | — | Connect to a network protected by 802.1X, view 802.1X profile settings, and enable automatic connection. | — |
| MD.06.c087 | Details button settings | Proxies | other | — | — | Configure internet proxies. | — |
| MD.06.c088 | Details button settings | Hardware | other | — | — | Configure hardware settings and view the hardware MAC address. | — |
| MD.06.c089 | Advanced button settings | Change networks | checkbox | Enabled; Disabled | — | Require an administrator password to switch to a different Wi-Fi network. | Switching to a different Wi-Fi network requires an administrator password when enabled. |
| MD.06.c090 | Advanced button settings | Turn Wi-Fi on or off | checkbox | Enabled; Disabled | — | Require an administrator password to turn Wi-Fi on or off. | Turning Wi-Fi on or off requires an administrator password when enabled. |
| MD.06.c091 | Advanced button settings | Show legacy networks and options | checkbox | Enabled; Disabled | — | Display legacy networks. | — |
| MD.06.c092 | Advanced button settings | Wi-Fi MAC address | info | — | — | Shows the media access control (MAC) address for the computer’s Wi-Fi interface. | — |
| MD.06.c093 | Advanced button settings | Known Networks | list | — | — | Lists Wi-Fi networks you’ve set up. The list provides actions to remove a network, set its auto-join option, or copy its password. | — |

Notes: The page text does not specify a macOS version. The page refers to additional Apple Support articles for details about some settings.

## 13.7 MD.07 VPN

Status: **PARTIAL**. Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13).


### VPN: Change VPN settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp1579/27/mac/27) · topic `mchlp1579` · path: Apple menu > System Settings > VPN · "Open … for me" link: yes

Use VPN settings to set up and manage a VPN connection. Available settings vary depending on the type of VPN service selected.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.07.c001 | VPN settings | *service name* | toggle | On or off | — | Shows the VPN service name and connection status. Turn the service on or off to connect or disconnect. Click to manage the service. | — |
| MD.07.c002 | VPN settings | Set up a new VPN service. | button | — | — | Set up a new VPN service. | — |
| MD.07.c003 | VPN Info button settings | Display Name | field | — | — | Enter a name for the service. | — |
| MD.07.c004 | VPN Info button settings | Server Address | field | — | — | Enter the address of the VPN server. | — |
| MD.07.c005 | VPN Info button settings | Account Name | field | — | — | Enter the account name. | — |
| MD.07.c006 | VPN Info button settings | Password | field | — | — | Enter the password for the VPN account. | Appears for Cisco IPSec VPN connections. |
| MD.07.c007 | VPN Info button settings | Remote ID | field | — | — | Enter the remote ID for the VPN account. | Appears for IKEv2 VPN connections. |
| MD.07.c008 | VPN Info button settings | Local ID | field | — | — | Enter the local ID for the VPN account. | Appears for IKEv2 VPN connections. |
| MD.07.c009 | VPN Info button settings | Authentication | other | — | — | Enter the authentication details for the account. | — |
| MD.07.c010 | VPN Info button settings | Connect on demand | other | — | — | Automatically start the VPN connection when needed. | — |
| MD.07.c011 | VPN Info button settings | Remove configuration | button | — | — | Remove the selected VPN service. | — |
| MD.07.c012 | VPN Info button settings > Options | Disconnect when switching user accounts | checkbox | — | — | End the VPN session when you switch to a different user account on your Mac. | Available for L2TP VPN services. |
| MD.07.c013 | VPN Info button settings > Options | Disconnect when user logs out | checkbox | — | — | End the VPN session when you log out. | Available for L2TP VPN services. |
| MD.07.c014 | VPN Info button settings > Options | Send all traffic over VPN connection | checkbox | — | — | Send all network traffic over the VPN connection, regardless of the network service you use to access the network (for example, Wi-Fi or Ethernet). | Available for L2TP VPN services. |
| MD.07.c015 | VPN Info button settings > Options | Use verbose logging | checkbox | — | — | Capture detailed log information during the VPN session. | Available for L2TP VPN services. |
| MD.07.c016 | VPN Info button settings | TCP/IP | link | — | — | Configure IPv4 and IPv6. See Change TCP/IP settings. | — |
| MD.07.c017 | VPN Info button settings | DNS | link | — | — | Configure DNS servers and search domains. See Change DNS settings. | — |
| MD.07.c018 | VPN Info button settings | Proxies | link | — | — | Configure internet proxies. See Change proxy settings. | — |

Notes: The available settings vary depending on the type of VPN service selected.

## 13.8 MD.08 Battery / Energy

Status: **PARTIAL**. Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13).


### Energy: Change Energy settings on a Mac desktop computer

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp1168/27/mac/27) · topic `mchlp1168` · path: Apple menu > System Settings > Energy · "Open … for me" link: no

Use Energy settings to control a Mac desktop computer’s energy use. Some options may not be available, depending on your Mac.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.08.c001 | Energy | Low Power Mode | toggle | — | — | Reduce energy usage for computers that are always on (such as those used as servers) and fan noise for environments and tasks that require minimal noise. | Some options may not be available, depending on your Mac. |
| MD.08.c002 | Energy | Prevent automatic sleeping when the display is off | toggle | — | — | Prevent your Mac from going to sleep automatically when its display is off. | Some options may not be available, depending on your Mac.; Delaying or preventing sleep may increase power consumption. |
| MD.08.c003 | Energy | Wake for network access | toggle | — | — | Even in sleep mode, wake your Mac to provide access to shared resources, such as shared printers or Music playlists. | Some options may not be available, depending on your Mac.; Allowing disruption of sleep may increase power consumption. |
| MD.08.c004 | Options | Start up when power is connected | popup | Never; After power failure; Always | — | Choose when to start up your Mac after power is connected. Never: Press the power button to turn on your Mac. After power failure: Turn on your Mac when reconnected to power if the computer was on before a power failure. Always: Turn on your Mac when reconnected to power. | Some options may not be available, depending on your Mac. |
| MD.08.c005 | Options | Start up automatically after a power failure | toggle | — | — | Start up your Mac after a power interruption. | Some options may not be available, depending on your Mac. |
| MD.08.c006 | Options | Enable Power Nap | toggle | — | — | Have your Mac check for email, calendar, and other iCloud updates while sleeping. | Some options may not be available, depending on your Mac. |
| MD.08.c007 | Options | UPS Options | button | — | — | Set the time or battery level at which your Mac shuts down when using a UPS. | When using a UPS. |

Notes: As an ENERGY STAR® partner, Apple says standard configurations of Mac meet the ENERGY STAR® guidelines for energy efficiency. Delaying, preventing, or allowing disruption of sleep may increase power consumption. To turn off your display after a period of inactivity, see Change Lock Screen settings.

### Battery: Change Battery settings on a Mac laptop

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlfc3b7879/27/mac/27) · topic `mchlfc3b7879` · path: Apple menu > System Settings > Battery · "Open … for me" link: no

Use Battery settings to control a Mac laptop’s battery and energy use and help optimize battery lifespan.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.08.c008 | Battery | Battery | info | — | — | Shows the current battery level and any battery alerts at the top. | Some options may not be available, depending on your Mac. |
| MD.08.c009 | Battery | Low Power Mode | other | — | — | Reduce energy usage to increase battery life. | Some options may not be available, depending on your Mac. |
| MD.08.c010 | Battery | Energy Mode | popup | Low Power; Automatic; High Power | — | Choose which energy mode your Mac uses while on battery power or plugged into a power adapter. Low Power reduces energy usage to increase battery life. Automatic has your Mac automatically use the best performance level. High Power increases energy usage to improve performance during sustained workloads. | Some options may not be available, depending on your Mac. High Power Mode is described in the Apple Support article “Use High Power Mode on your 14-inch or 16-inch MacBook Pro.” |
| MD.08.c011 | Battery | Battery Health | button | — | — | Click to see whether the battery’s condition is normal or service is recommended. | Some options may not be available, depending on your Mac. |
| MD.08.c012 | Battery Health | Battery Condition | info | — | — | Shows whether the battery’s condition is normal or service is recommended. | — |
| MD.08.c013 | Battery Health | Maximum Capacity | info | — | — | Shows the measure of the battery’s capacity relative to when it was new. | — |
| MD.08.c014 | Battery Health | Manage battery longevity | other | — | — | Reduce peak capacity to optimize the battery’s lifespan. | — |
| MD.08.c015 | Battery | Charging | other | — | — | Click to access charging options. | Some options may not be available, depending on your Mac. |
| MD.08.c016 | Charging | Charge Limit | other | — | — | Preserve the lifespan of your battery by limiting how much your battery charges to. | The page text says, “To charge the battery to full, click” but does not provide the item to click. |
| MD.08.c017 | Charging | Optimized Battery Charging | other | — | — | Have your Mac learn your daily charging routine to reduce battery aging. Your Mac delays charging past 80% when it predicts that you’ll be plugged in for an extended period of time, and aims to fully charge the battery before you unplug. | — |
| MD.08.c018 | Last 24 Hours | Battery Level | info | — | — | Shows the average charge level of your battery for each 15-minute period. Shaded areas indicate when your Mac was charging. | — |
| MD.08.c019 | Last 24 Hours | Screen On Usage | info | — | — | Shows how long your screen was turned on each hour or each day. | — |
| MD.08.c020 | Last 10 Days | Energy Usage | info | — | — | Shows how much energy your Mac used each day. | — |
| MD.08.c021 | Last 10 Days | Screen On Usage | info | — | — | Shows how long your screen was turned on each hour or each day. | — |
| MD.08.c022 | Additional Options | Slightly dim the display on battery | other | — | — | Conserve energy by dimming the display when your Mac is not plugged in. | — |
| MD.08.c023 | Additional Options | Prevent automatic sleeping on power adapter when the display is off | other | — | — | Prevent your Mac from sleeping when it’s plugged in and its display is off. | — |
| MD.08.c024 | Additional Options | Put hard disks to sleep when possible | other | — | — | Put the hard drive to sleep whenever the computer is inactive. | — |
| MD.08.c025 | Additional Options | Wake for network access | popup | Always; when plugged into a power adapter; never | — | Even in sleep mode, wake your Mac to provide access to shared resources, such as shared printers or Music playlists. | — |
| MD.08.c026 | Additional Options | Automatic graphics switching | other | — | — | To optimize the lifespan of your battery, have your Mac automatically choose the best graphics mode based on your usage. | — |
| MD.08.c027 | Additional Options | Optimize video streaming while on battery | other | — | — | While on battery power, play high-dynamic-range (HDR) video in standard dynamic range (SDR), which requires less energy. | — |

Notes: The page says some options may not be available, depending on your Mac. The page notes that ENERGY STAR standard configurations of Mac meet ENERGY STAR guidelines for energy efficiency. The page refers to Change Lock Screen settings for turning the display off after a period of inactivity. No macOS version is stated in the supplied page text.

#### Independent macOS 27 reports for MD.08

| ID | Kind | Pane | Claim | Source |
|---|---|---|---|---|
| MD.08.x01 | changed | Battery | The single Low Power Mode menu was replaced by separate menus for battery and power-adapter use. Each offers Automatic, Low Power, or High Power. | T4 [geeksmodo](https://geeksmodo.com/macos-system-settings/) |
| MD.08.x02 | identifier | Desktop & Dock, Displays, Wallpaper, Screen Saver, Battery, and Energy Saver | The listed commands open these sections. Displays and Battery each have an alternate legacy command; Energy Saver is listed with a preference-pane command. | T2 [GitHub](https://github.com/bvanpeski/SystemPreferences/blob/main/macos_preferencepanes-Ventura.md) |
| MD.08.x03 | behavior | Battery | The article describes battery settings for sleep, Power Nap, and balancing performance with battery life under System Settings > Battery. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.08.x04 | behavior | Battery | The article says Optimized Battery Charging is enabled by clicking the information button next to Charging and toggling the option on. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.08.x05 | new | Battery | On macOS 27, the charging sheet also has a Charge Limit slider from 80 to 100%; the article says setting it to 80% can help reduce battery aging. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |

## 13.9 MD.09 General (About, Software Update, Storage, AppleCare, AirDrop & Handoff, Login Items & Extensions, Language & Region, Date & Time, Sharing, Time Machine, Transfer or Reset, Startup Disk, Device Management)

Status: **PARTIAL**. 12 of 13 General subpages from the 27 guide; AppleCare & Warranty and AutoFill & Passwords only from support articles (GAP-B4-11).


### General > About: View About settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlea7173f3/27/mac/27) · topic `mchlea7173f3` · path: Apple menu > System Settings > General > About · "Open … for me" link: no

About settings shows details about your Mac, including its hardware, macOS version, warranty and repair coverage, and links to a detailed system report and other information.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.09.c001 | About | Name | field | — | — | Shows the name of your Mac. You can select and change it. | — |
| MD.09.c002 | About | Chip (Apple silicon) or Processor (Intel) | info | — | — | Shows the type of processor chip in your Mac. | The label depends on whether your Mac has Apple silicon or an Intel processor. |
| MD.09.c003 | About | Graphics (Intel only) | info | — | — | Shows the type of graphic chip in your Mac. | Intel Mac only. |
| MD.09.c004 | About | Memory | info | — | — | Shows the amount of total memory in your Mac. | — |
| MD.09.c005 | About | Serial number | info | — | — | Shows the serial number of your Mac. | — |
| MD.09.c006 | About | Coverage | info | — | — | Identifies the warranty and coverage information area. | — |
| MD.09.c007 | Coverage | Details | button | — | — | Opens the status of your warranty and coverage and provides access to support options. | — |
| MD.09.c008 | About | macOS | info | — | — | Shows the version of macOS on your Mac. | — |
| MD.09.c009 | About | Displays | info | — | — | Identifies the display information area. | — |
| MD.09.c010 | Displays | Display Settings | button | — | — | Opens settings selected for your display. | — |
| MD.09.c011 | About | Storage | info | — | — | Identifies the storage information area. | — |
| MD.09.c012 | Storage | Storage Settings | button | — | — | Opens options for optimizing the amount of storage available on your Mac. | — |
| MD.09.c013 | About | System Report | button | — | — | Opens an in-depth technical listing of your hardware, network, and software features. | — |
| MD.09.c014 | About | Software License Agreement | link | — | — | Opens information about the license agreements, certification, and ENERGY STAR compliance statement for your Mac. | — |
| MD.09.c015 | About | Regulatory Certification | link | — | — | Opens information about the license agreements, certification, and ENERGY STAR compliance statement for your Mac. | — |
| MD.09.c016 | About | ENERGY STAR® Compliance | link | — | — | Opens information about the license agreements, certification, and ENERGY STAR compliance statement for your Mac. | — |
| MD.09.c017 | About | License Agreement | link | — | — | Opens information about the license agreements, certification, and ENERGY STAR compliance statement for your Mac. | — |

### General > AirDrop & Handoff: Change AirDrop & Continuity settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl6a407f99/27/mac/27) · topic `mchl6a407f99` · path: System Settings > General > AirDrop & Continuity · "Open … for me" link: yes

Use AirDrop & Continuity settings to use Handoff between Apple devices, choose who can send files to your Mac with AirDrop, and allow devices to stream content to your Mac with AirPlay.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.09.c018 | AirDrop & Continuity | Allow Handoff between this Mac and your iCloud devices(Not available on all computers) | toggle | — | — | Turns on Handoff between your Mac, iPhone, iPad, and Apple Watch. When selected, you can copy and paste content between your Apple devices. | Available only on computers that support Handoff. Your Apple devices must have Wi-Fi, Bluetooth, and Handoff turned on in Settings, and be signed in to the same Apple Account. |
| MD.09.c019 | AirDrop & Continuity | AirDrop | popup | Contacts Only; Everyone | — | Choose who can send files to you from nearby Apple devices. | Contacts Only is available on devices with iOS 10, iPadOS 13, macOS 10.12, or later. On devices with earlier software, select Everyone to receive AirDrop items. |
| MD.09.c020 | AirDrop & Continuity | AirPlay Receiver | toggle | — | — | Allows nearby Apple devices to stream content to this Mac with AirPlay. | Available on supported Mac computers with macOS 12 or later. May be unavailable if the Mac is enrolled in a mobile device management (MDM) system. |
| MD.09.c021 | AirDrop & Continuity | Allow AirPlay for | popup | Current User; Anyone on the Same Network; Everyone | — | Choose whose devices can use AirPlay to stream content to this Mac. Current User: Only devices signed in to your Apple Account can see and use AirPlay. Anyone on the Same Network: Any nearby Mac, iPhone, or iPad on the same network can see and use AirPlay. Everyone: Any Mac, iPhone, or iPad can see and use AirPlay. | — |
| MD.09.c022 | AirDrop & Continuity | Require password | toggle | — | — | Requires a password to use AirPlay to stream content to this Mac. | When turned on, click Set, enter a password in the text field, then click OK. |
| MD.09.c023 | AirDrop & Continuity | Set | button | — | — | Opens the password entry step for AirPlay. | Used after turning on Require password. |
| MD.09.c024 | AirDrop & Continuity | Password text field | field | — | — | Enter the password required to use AirPlay to stream content to this Mac. | Shown after clicking Set. |
| MD.09.c025 | AirDrop & Continuity | OK | button | — | — | Confirms the entered AirPlay password. | Used after entering a password in the text field. |

Notes: The page says the Everyone option for AirDrop can be turned off after receiving items.

### General > Transfer or Reset: Use Transfer or Reset settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchla14c784b/27/mac/27) · topic `mchla14c784b` · path: Apple menu > System Settings > General > Transfer or Reset · "Open … for me" link: no

Use Transfer or Reset settings to open Migration Assistant and transfer information to your Mac, or open Erase Assistant and erase all content and settings.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.09.c026 | Transfer or Reset | Open Migration Assistant | button | — | — | Opens Migration Assistant to transfer files from another device to your Mac. | Opens Migration Assistant. |
| MD.09.c027 | Transfer or Reset | Erase All Content and Settings | button | — | — | Opens Erase Assistant to erase all files and settings on your Mac. | Opens Erase Assistant to safely erase all content and settings on your Mac. |

### General > Software Update: Software Update settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchla7037245/27/mac/27) · topic `mchla7037245` · path: Apple menu · "Open … for me" link: yes

Use Software Update settings to install available system software and other updates, choose automatic update options, and manage beta updates.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.09.c028 | Available Updates | Update Now | button | — | — | Immediately installs all available system software and other updates shown. | Installs the selected available updates. |
| MD.09.c029 | Available Updates | Update Tonight | button | — | — | Installs downloaded updates overnight. | The Mac must be turned on, connected to power, and connected to the internet overnight.; Installs downloaded updates overnight. |
| MD.09.c030 | Available Updates | Update | list | Available system software and other updates | — | Select an update to show details about it. | — |
| MD.09.c031 | Available Updates | Updates you want | checkbox | — | — | Select the checkboxes for the updates to install, then click Update Now. | Installs the selected updates when Update Now is clicked. |
| MD.09.c032 | Automatic Updates | Download new updates when available: | other | — | — | Automatically downloads macOS updates without asking. | Downloads macOS updates automatically. |
| MD.09.c033 | Automatic Updates | Install macOS updates: | other | — | — | Installs macOS updates automatically. | Installs macOS updates automatically. |
| MD.09.c034 | Automatic Updates | Install Security Responses and system files: | other | — | — | Automatically installs system files and Rapid Security Responses. | Installs system files and Rapid Security Responses automatically. |
| MD.09.c035 | Beta Updates | Beta Updates | other | Turn off beta updates, or choose the type of beta to receive. | — | Turns off beta updates or chooses the type of beta to receive. | Available only if you’re enrolled in the Apple Beta Software Program. |

Notes: If an update button isn’t shown, your Mac is up to date.

### General > Date & Time: Change Date & Time settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp1124/27/mac/27) · topic `mchlp1124` · path: Apple menu > System Settings > General > Date & Time · "Open … for me" link: yes

Use Date & Time settings to set or change the date, time, time zone, and 24-hour time display on your Mac.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.09.c036 | Date & Time options | Set time and date automatically | toggle | On or off; when setting up automatic time, click Set and enter the address of a network time server. | — | Have the date and time on your Mac set automatically using a network time server. | If the Mac isn’t connected to the internet, it connects when you choose this option. |
| MD.09.c037 | Date & Time options | Date and time | other | — | — | Shows the date and time automatically set on your Mac. To set them manually, turn off “Set date and time automatically,” click Set, enter the date and time, then click Set. | — |
| MD.09.c038 | Date & Time options | 24-hour time | toggle | On or off. | — | Use a 24-hour format for the time shown in the menu bar. For example, instead of 3:00 PM, show 15:00. | — |
| MD.09.c039 | Date & Time options | Set time zone automatically using your current location | toggle | On or off. | — | Have the time zone on your Mac set automatically based on your current location. | Location Services must be turned on to determine your current location. |
| MD.09.c040 | Date & Time options | Time zone | other | — | — | Shows the time zone automatically set on your Mac. To set the time zone manually, turn off “Set time zone automatically using your current location,” enter a city in the bottom text field, then press Return. | — |
| MD.09.c041 | Date & Time options | Closest city | info | — | — | A major city in your desired time zone. To change the city, turn off “Set time zone automatically using your current location,” enter a city in the text field, then press Return. | — |

Notes: The page says to choose Apple menu > System Settings to change these settings. The page refers to changing how dates and times appear in other languages and to changing Menu Bar settings. Tip: Click the date and time in the menu bar to open Notification Center.

### General > Language & Region: Change Language & Region settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/intl163/27/mac/27) · topic `intl163` · path: Apple menu > System Settings > General > Language & Region · "Open … for me" link: yes

Use Language & Region settings to choose the languages shown in macOS and apps, and the formats used for dates, times, numbers, and more.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.09.c042 | Language & Region | Preferred Languages | list | Choose one or more languages. | — | Choose the language or languages shown in macOS and apps, and on websites when available in a preferred language. | If you add input sources, those languages are automatically added to the list. |
| MD.09.c043 | Language & Region | Settings pop-up menu | popup | Apply to Login Window | — | Apply the primary language from Preferred Languages to the login window. | If the Mac has multiple users, everyone sees the login window in this language. If the menu isn’t shown, the login window is already set to use the primary language. |
| MD.09.c044 | Language & Region | Region | popup | — | — | Choose the geographic region whose formats are used for dates, times, numbers, and currencies. | The options and preview shown change when you choose a different region. |
| MD.09.c045 | Language & Region | Calendar | popup | — | — | Choose the type of calendar used to display dates and times. | — |
| MD.09.c046 | Language & Region | Temperature | popup | — | — | Select the preferred temperature format. | — |
| MD.09.c047 | Language & Region | Measurement system | popup | — | — | Select the preferred measurement system. | — |
| MD.09.c048 | Language & Region | First day of week | popup | — | — | Choose the first day of the week. | — |
| MD.09.c049 | Language & Region | Date format | popup | — | — | Choose the date format, such as day, then month, then year. | — |
| MD.09.c050 | Language & Region | Number format | popup | — | — | Choose the format for numbers, such as when a comma appears. | — |
| MD.09.c051 | Language & Region | List sort order | popup | — | — | Choose the sort order for names. | Available only for some primary languages. |
| MD.09.c052 | Language & Region | Term of address | other | feminine, masculine, or neutral | — | In supported languages, personalize how your Mac addresses you by choosing your preferred term of address. | Available in supported languages. |
| MD.09.c053 | Language & Region | Share with all applications | toggle | — | — | Turn on to let all apps personalize how they address you. | — |
| MD.09.c054 | Language & Region | Live Text | other | — | — | Select text in images to copy or take action. | — |
| MD.09.c055 | Language & Region | Applications | other | — | — | Customize language settings for individual applications. | — |
| MD.09.c056 | Language & Region | Translation Languages | other | — | — | Select languages to download to your computer so you can translate when offline. You can also process translations on your Mac when connected to the internet. | Selected languages can be used for offline translation. |

### General > Storage: Change Storage settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl3d437fbc/27/mac/27) · topic `mchl3d437fbc` · path: Apple menu > System Settings > General > Storage · "Open … for me" link: no

View storage use and available space, review storage recommendations, and inspect or remove files by category. Some categories only show how much space they use and can’t be managed in Storage settings.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.09.c057 | Storage | Storage bar chart | info | — | — | Shows used and available storage on your Mac, including the amount used by apps and file types and the amount of free storage. | — |
| MD.09.c058 | Storage | All Volumes | button | — | — | Shows a more detailed bar chart of the storage space used and available for each volume and container of your internal hard disk. | — |
| MD.09.c059 | Storage recommendations | Store in iCloud | button | — | — | Turns on iCloud Drive and stores most files in iCloud, saving storage space while keeping only recent files in their entirety on the Mac. | Turns on iCloud Drive. |
| MD.09.c060 | Storage recommendations | Optimize Storage | button | — | — | Automatically removes movies and TV shows that you’ve already viewed from your Mac. | Removes viewed movies and TV shows from the Mac. |
| MD.09.c061 | Storage recommendations | Empty Trash automatically | checkbox | — | — | Sets your Mac to automatically remove files that have been in the Trash for more than 30 days. | Files in the Trash for more than 30 days are automatically removed. |
| MD.09.c062 | Categories by file type | Categories by file type | list | Applications; Documents; iCloud Drive; iOS files; TV; Music; Books; Podcasts; Mail; Messages; Music Creation; Photos; Trash | — | Select a category to review files. Applications contains installed apps other than system applications. Documents includes files in your home folder not included in other categories, including some documents, videos, and photos; it provides ways to view large files or downloads and sort files by Name, Kind, Last Accessed, or Size, and a File Browser. iCloud Drive includes files stored there and may include Desktop and Documents files if those folders are stored in iCloud Drive. iOS files includes iOS backups and firmware. TV, Music, Books, and Podcasts contain purchases from their respective apps. Mail contains emails and attachments; Messages contains conversation content and attachments; Music Creation contains GarageBand, Logic, and MainStage content; Photos contains the System Photo Library; and Trash contains deleted items. | — |
| MD.09.c063 | Categories by file type | Delete | button | — | — | Deletes selected files from an open category. | Select an unneeded file in a category.; Deletes the selected file. |
| MD.09.c064 | Categories by file type | Show in Finder | button | — | — | Shows the selected file in Finder. | Select a file in a category. |
| MD.09.c065 | Documents | Large files and downloads | button | — | — | Opens a view of large files or downloads. | — |
| MD.09.c066 | Documents | Name; Kind; Last Accessed; Size | other | — | — | Sorts files by the selected column. | — |
| MD.09.c067 | Documents | File Browser | button | — | — | Shows the contents and amount of storage used by various folders in the file system. | — |
| MD.09.c068 | Other Users, macOS, and System Data | Other Users & Shared | info | — | — | Shows the amount of space used by files created and modified, or shared, by other user accounts on the Mac. Files in this category can’t be selected and deleted here. | — |
| MD.09.c069 | Other Users, macOS, and System Data | macOS | info | — | — | Shows the amount of space used by macOS system applications and files. Files in this category can’t be selected and deleted here. | — |
| MD.09.c070 | Other Users, macOS, and System Data | System Data | info | — | — | Shows the amount of space used by files that don’t fall into the other categories, such as logs, caches, VM files, runtime resources, temporary files, fonts, app support files, and plug-ins. Its contents are managed by macOS and can’t be managed in Storage settings. | — |

Notes: The page does not state a macOS version. Files deleted from iCloud Drive aren’t included in the Trash storage amount. Purchases in the TV, Music, Books, and Podcasts categories can be removed from the Mac and downloaded again. Data outside the listed categories can be managed using Finder or the third-party apps that created it.

### General > Startup Disk: Change Startup Disk settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl316386f7/27/mac/27) · topic `mchl316386f7` · path: Apple menu > System Settings > General > Startup Disk · "Open … for me" link: yes

Use Startup Disk settings to choose a disk to start your Mac from, including a network volume, a different disk, or another operating system.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.09.c071 | Startup Disk settings | Startup Disks | list | Disks available to start your Mac; options are not individually listed. | — | Shows the disks you can use to start your Mac. Click to select the startup disk you want to use. | On a Mac with an Apple T2 Security Chip, added security features may need to be set to change the startup disk. On a Mac with Apple silicon, see Change the security policy. |
| MD.09.c072 | Startup Disk settings | Restart | button | — | — | Restart your Mac using the selected startup disk. | Restarts your Mac. |
| MD.09.c073 | Startup Disk settings | Restart in Target Disk Mode (Mac with Intel chip only) | button | — | — | Restarts your Mac in target disk mode, so it can appear as an external hard disk on another Mac when the computers are connected using USB, USB-C, or Thunderbolt. | Mac with Intel chip only.; Restarts your Mac in Target Disk Mode. |

Notes: When selecting a network startup volume, select a network startup volume, not a network install image. Selecting a network install image reinstalls system software and may erase the contents of the disk. A standard network volume icon appears as a globe with a folder. A network install icon appears as a globe with a downward-pointing green arrow. The page refers users with Apple silicon to Use macOS Recovery on a Mac with Apple silicon instead of Target Disk Mode.

### General > Sharing: Change Sharing settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl26e04309/27/mac/27) · topic `mchl26e04309` · path: Apple menu > System Settings > General > Sharing · "Open … for me" link: yes

Use Sharing settings to set options for sharing your computer, files, printers, media, and more.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.09.c074 | Sharing | File Sharing | other | — | — | Allow other users to access shared folders on your Mac, and allow administrators to access all volumes. Click See Set up file sharing. | Some sharing settings may be unavailable if your Mac is enrolled in a device management service. |
| MD.09.c075 | Sharing | Media Sharing | other | — | — | Allow other devices on your network to browse and play downloaded music, movies, and TV shows from your Mac. Click See Change Media Sharing settings. | Some sharing settings may be unavailable if your Mac is enrolled in a device management service. |
| MD.09.c076 | Sharing | Screen Sharing | other | — | — | Allow other users on your network to access your computer screen. Click See Turn Mac screen sharing on or off. | Some sharing settings may be unavailable if your Mac is enrolled in a device management service. |
| MD.09.c077 | Sharing | Content Caching | other | — | — | Reduce bandwidth usage and speed up installation on supported devices by storing downloaded software updates, applications, and other content on your Mac. Click See Change Content Caching settings. | Some sharing settings may be unavailable if your Mac is enrolled in a device management service. |
| MD.09.c078 | Sharing | Bluetooth Sharing | other | — | — | Allow Bluetooth® file sharing. Click See Set up Bluetooth sharing. | Some sharing settings may be unavailable if your Mac is enrolled in a device management service. |
| MD.09.c079 | Sharing | Printer Sharing | other | — | — | Allow others on your network to use printers connected to your Mac. Click See Share your printer. | Some sharing settings may be unavailable if your Mac is enrolled in a device management service. |
| MD.09.c080 | Sharing | Internet Sharing | other | — | — | Allow other devices to share your connection to the internet. Click See Share the internet connection on Mac with other network users. | Some sharing settings may be unavailable if your Mac is enrolled in a device management service. |
| MD.09.c081 | Sharing | Remote Management | other | — | — | Allow others to access your Mac using Apple Remote Desktop. Click See Allow Apple Remote Desktop to access your Mac. | Some sharing settings may be unavailable if your Mac is enrolled in a device management service. |
| MD.09.c082 | Sharing | Remote Login | other | — | — | Allow remote login access to your Mac. Click See Allow a remote computer to access your Mac. | Some sharing settings may be unavailable if your Mac is enrolled in a device management service. |
| MD.09.c083 | Sharing | Remote Application Scripting | other | — | — | Allow Apple events sent from other computers to control applications on this Mac. Click See Allow remote application scripting. | Some sharing settings may be unavailable if your Mac is enrolled in a device management service. |
| MD.09.c084 | Sharing | Local hostname | info | — | — | The local network name of your Mac. | — |
| MD.09.c085 | Sharing | Edit | button | — | — | Click Edit to change your computer’s local hostname or use a dynamic global hostname. | — |

Notes: If your Mac is enrolled in a device management service, some sharing settings may be unavailable.

### General > Device Management: Change Device Management settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mh35474/27/mac/27) · topic `mh35474` · path: Apple menu > Device Management settings · "Open … for me" link: yes

Use Device Management settings to install, view, or remove configuration profiles. Profiles standardize settings and may be required to access network, email, VPN, or other services.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.09.c086 | Device Management settings | Add a profile | button | — | — | Click the add control to add a configuration profile. | Some profiles can only be installed within a specified time period. |
| MD.09.c087 | Device Management settings | Delete a profile | button | — | — | Select a profile, then click the delete control to remove it. | The Mac may be managed; some profiles can only be removed by a system administrator. |
| MD.09.c088 | Device Management settings | Select the profile | list | — | — | Select a profile before deleting it. | — |

Notes: The page does not state a macOS version. If a profile cannot be removed, contact the system administrator of the school or institution that provided the Mac. A system administrator may ask you to install a profile to access network, email, VPN, or other services.

### General > Time Machine: Time Machine settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mh14037/27/mac/27) · topic `mh14037` · path: Apple menu · "Open … for me" link: no

Use Time Machine settings to add or remove a backup disk and customize backup frequency, battery-powered backups, and items excluded from backups.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.09.c089 | Options | Add Backup Disk | button | — | — | Set up a backup disk if you haven’t set up Time Machine. | Shown if you haven’t set up Time Machine. |
| MD.09.c090 | Add or Remove Backup Disk | Add a backup disk | other | — | — | Click the add control to add a backup disk. | For a Mac that already has Time Machine set up. |
| MD.09.c091 | Add or Remove Backup Disk | Forget Destination | button | — | — | Remove a backup disk by clicking the remove control, then clicking “Forget Destination.” | For a Mac that already has Time Machine set up. |
| MD.09.c092 | Options | Options | button | — | — | Open the backup customization options. | — |
| MD.09.c093 | Options | Backup Frequency | popup | Back up manually; select how often Time Machine backs up files automatically. | — | Choose whether to back up manually or how often Time Machine backs up files automatically. | — |
| MD.09.c094 | Options | Back up on battery power | toggle | — | — | Turn on to create backups every hour even if your Mac is on battery power. | Appears only when an automatic backup schedule is selected on a Mac laptop. |
| MD.09.c095 | Options | Exclude from Backups | button | — | — | Choose files and folders to leave out of the backup. | — |

Notes: The supplied page text only shows “To change these settings, choose Apple menu” and does not provide the rest of the navigation path. The page does not specify a macOS version. If you create a backup on a Mac, you can only use that backup on Mac computers. If you partition with macOS 12 or later, you can only use the partitions with macOS 11 or later.

### General > Login Items & Extensions: Change Login Items & Extensions settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mtusr003/27/mac/27) · topic `mtusr003` · path: Apple menu > System Settings > General > Login Items & Extensions · "Open … for me" link: no

Administrators can choose items that open at login, allow apps to perform background tasks, and manage Apple and third-party extensions, including which extensions appear in the Share menu.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.09.c096 | Login Items settings | Add a new login item | button | Click the add (+) button. | — | Adds an app, document, folder, or connection to the items that open automatically when you log in. | You must be an administrator of the Mac to change these settings. |
| MD.09.c097 | Login Items settings | Delete a login item | button | Select an item in the login items list, then click the delete (−) button. | — | Removes the selected item from the login items list. | You must be an administrator of the Mac to change these settings. |
| MD.09.c098 | Login Items settings | Item column | info | — | — | Shows the names of apps, documents, folders, or connections that open automatically. | — |
| MD.09.c099 | Login Items settings | Kind column | info | — | — | Shows the type of item (app, document, folder, volume). | — |
| MD.09.c100 | Login Items settings | App Background Activity | toggle | — | — | Allows apps to perform tasks when the app isn’t open, such as checking for software updates or syncing data. | — |
| MD.09.c101 | Extensions settings | Added extensions | list | For content extensions, an Actions checkbox is shown below the extension. For Finder extensions, a Finder checkbox is shown. | — | Lists extensions installed on your Mac that were created by third-party developers. Select the applicable checkbox to enable the extension. | — |
| MD.09.c102 | Extensions settings > Added extensions | Actions | checkbox | — | — | Select a content extension to make it available in apps. For example, when Markup is turned on, you can draw on pictures and sign documents in apps such as Mail, TextEdit, or Preview. | Shown below a content extension. |
| MD.09.c103 | Extensions settings > Added extensions | Finder | checkbox | — | — | Enables a Finder extension. | Shown below a Finder extension. |
| MD.09.c104 | Extensions settings | Camera Extensions | list | — | — | Lists camera extensions installed on your Mac. These allow you to install and use custom cameras. | — |
| MD.09.c105 | Extensions settings | Color Panels | list | — | — | Lists color panel extensions installed on your Mac. These allow you to select and view colors, copy and paste color codes, create palettes, and more. | — |
| MD.09.c106 | Extensions settings | Dock Tiles | list | — | — | Lists dock tile extensions installed on your Mac. These allow you to customize the Dock by adding spacers, set quick actions, see previews and actions, and more. | — |
| MD.09.c107 | Extensions settings | Drivers | list | — | — | Lists driver extensions installed on your Mac. These allow you to install custom drivers to give apps access to system functionalities. | — |
| MD.09.c108 | Extensions settings | Endpoint Security Extensions | list | — | — | Lists endpoint security extensions installed on your Mac. These enhance security functionality by monitoring system events for potentially malicious activity. | — |
| MD.09.c109 | Extensions settings | File Providers | list | — | — | Lists file provider extensions installed on your Mac. These allow you to see local files and remote storage files in one view in Finder. | — |
| MD.09.c110 | Extensions settings | Finder | list | — | — | Lists Finder extensions installed on your Mac, such as iCloud Drive. Finder extensions enhance search and organization for files, documents, images, and more. Turn on extensions you want to access from the Quick Actions menu on the desktop, and in Finder and the Finder Preview pane. | — |
| MD.09.c111 | Extensions settings | Media | list | — | — | Lists media extensions installed on your Mac. These add controls for playback, editing, and sharing, and can also be used for playing various formats. | — |
| MD.09.c112 | Extensions settings | Network Extensions | list | — | — | Lists network extensions installed on your Mac. These allow you to customize and extend core networking features, such as configuring a VPN or content filter. | — |
| MD.09.c113 | Extensions settings | Photo Projects | list | — | — | Lists photo project extensions installed on your Mac. These allow you to create, edit, and order print projects in Photos. | — |
| MD.09.c114 | Extensions settings | Photos Editing | list | — | — | Lists Photos extensions installed on your Mac, such as Markup. Turn on an extension to make it available in Photos while editing photos. | — |
| MD.09.c115 | Extensions settings | Quick Look | list | — | — | Lists Quick Look extensions installed on your Mac. These allow you to click a file, then press the space bar to preview it. | — |
| MD.09.c116 | Extensions settings | Sharing | checkbox | Sharing extensions to include in the Share menu. Default extensions, such as Mail and AirDrop, can’t be turned off. | — | Select the sharing extensions to include in the Share menu. An enabled extension might not appear for an item that can’t be shared using that extension; for example, AirDrop can share a webpage in Safari but can’t share selected text. | — |
| MD.09.c117 | Extensions settings | Smart Card Readers | list | — | — | Lists smart card reader extensions installed on your Mac. These allow you to manage smart cards and certificates, authenticate, and complete tasks across apps. | — |
| MD.09.c118 | Extensions settings | Spotlight | list | — | — | Lists Spotlight extensions installed on your Mac. These enhance search functionality by allowing your Mac to search and display content from many sources and apps. | — |
| MD.09.c119 | Extensions settings | Touch Bar | checkbox | — | — | Turn on extensions you want to access from the Touch Bar. | Only if your Mac has a Touch Bar. Touch Bar options must first be set to show Quick Actions, or the Quick Actions button must be added to the Touch Bar. |

Notes: The page says you must be an administrator of your Mac to change these settings. The source text does not state a macOS version. The source text does not specify default states for the controls.

### General > AutoFill & Passwords: Use the Passwords app to create, manage, and share passwords and passkeys across Apple devices

T1 [Apple Support](https://support.apple.com/en-us/120758) · topic `120758` · path: System Settings > General > AutoFill & Passwords; System Settings > [your name] > iCloud > Passwords and Keychain; Passwords > All > [website or app] > Security > Set Up Verification Code... · "Open … for me" link: no

The Passwords app stores and manages passwords, passkeys, Wi-Fi passwords, and verification codes; it can autofill credentials, sync them with iCloud Keychain, share credentials, and identify common password weaknesses and known data leaks.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.09.c120 | Turn on Password AutoFill | General | other | — | — | Select General in Settings or System Settings. | — |
| MD.09.c121 | Turn on Password AutoFill | AutoFill & Passwords | other | — | — | Select AutoFill & Passwords. | — |
| MD.09.c122 | Turn on Password AutoFill | AutoFill Passwords and Passkeys | toggle | — | — | Turn on to have the device automatically enter passwords or passkeys saved in Passwords. | — |
| MD.09.c123 | Turn on Password AutoFill | extension | other | — | — | Enable an extension to use AutoFill in supported third-party browsers on Mac. | Mac; supported third-party browsers. Extensions are found in the browser’s web store. |
| MD.09.c124 | Turn on iCloud Keychain | [your name] | other | — | — | Select [your name] in Settings or System Settings. | Use this path if iCloud Keychain wasn’t turned on when the device was first set up. |
| MD.09.c125 | Turn on iCloud Keychain | iCloud | other | — | — | Select iCloud. | Use this path if iCloud Keychain wasn’t turned on when the device was first set up. |
| MD.09.c126 | Turn on iCloud Keychain | Passwords and Keychain | other | — | — | Select Passwords and Keychain. | Use this path if iCloud Keychain wasn’t turned on when the device was first set up. |
| MD.09.c127 | Turn on iCloud Keychain | iCloud Keychain | toggle | — | — | Turn on to make passwords, passkeys, and other account information available and up to date across approved Apple devices. | — |
| MD.09.c128 | Turn on iCloud Keychain | iCloud Keychain | other | — | — | Approve a new device using iCloud Keychain on another device or with an old device passcode. | When setting up a new device.; After approval, iCloud Keychain begins updating automatically on the new device. |
| MD.09.c129 | Set up automatic verification codes | All | list | — | — | In Passwords, select All, then select a website or app. | — |
| MD.09.c130 | Set up automatic verification codes | Security | other | — | — | Choose Security under a website or app entry. | — |
| MD.09.c131 | Set up automatic verification codes | Set Up Verification Code... | other | — | — | Choose this option to set up an automatic verification code. | For websites and apps that offer two-factor or multi-factor authentication. |
| MD.09.c132 | Security recommendations in Passwords | Security | list | — | — | In Passwords, choose Security and select an account with a weak or compromised password. | — |

Notes: The page says the Passwords app features start with iOS 18, iPadOS 18, macOS Sequoia, and visionOS 2, and recommends updating all Apple devices to the latest software version. The page is about the Passwords app and related settings, not a dedicated System Settings pane. No default values, restart requirements, confirmation dialogs, or authentication prompts are stated. The page describes generating and saving strong passwords, sharing passwords and passkeys through Shared Groups or AirDrop, sharing Wi-Fi passwords, and security recommendations, but does not specify additional control labels or settings for those actions. The page also mentions iCloud Passwords on Windows using iCloud for Windows, Google Chrome, or Microsoft Edge.

### General > AppleCare & Warranty: Find information about your warranty or AppleCare plan - Apple Support

T1 [Apple Support](https://support.apple.com/en-us/102607) · topic `102607` · path: Apple menu > System Settings > General > AppleCare & Warranty · "Open … for me" link: no

Explains how to check AppleCare coverage, find its expiration date, and get an agreement number or proof of coverage.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.09.c133 | Check if your device is covered by an AppleCare plan | AppleCare & Warranty settings | other | — | — | Open AppleCare & Warranty settings on your device to find coverage information. | The page says you might be able to open these settings on your device. |
| MD.09.c134 | Check if your device is covered by an AppleCare plan | Settings | link | — | — | Open Settings as part of checking AppleCare coverage. | — |
| MD.09.c135 | Check if your device is covered by an AppleCare plan | General | link | — | — | Select General in Settings. | — |
| MD.09.c136 | Check if your device is covered by an AppleCare plan | AppleCare & Warranty | link | — | — | Select AppleCare & Warranty to find coverage information. | — |
| MD.09.c137 | Check if your device is covered by an AppleCare plan | check your coverage online | link | — | — | Check coverage online to find your plan. | — |
| MD.09.c138 | Find your AppleCare coverage expiration date | Settings | link | — | — | On iPhone or iPad, go to Settings. | On iPhone or iPad. |
| MD.09.c139 | Find your AppleCare coverage expiration date | General | link | — | — | Tap General. | On iPhone or iPad. |
| MD.09.c140 | Find your AppleCare coverage expiration date | AppleCare & Warranty | link | — | — | Tap AppleCare & Warranty. | On iPhone or iPad. |
| MD.09.c141 | Find your AppleCare coverage expiration date | your device | list | — | — | Tap your device to see its AppleCare coverage expiration date. | On iPhone or iPad. |
| MD.09.c142 | Find your AppleCare coverage expiration date | Apple menu | other | — | — | Click the Apple menu in the corner of the screen. | On Mac. |
| MD.09.c143 | Find your AppleCare coverage expiration date | System Settings | link | — | — | Choose System Settings. | On Mac. |
| MD.09.c144 | Find your AppleCare coverage expiration date | General | link | — | — | Click General. | On Mac. |
| MD.09.c145 | Find your AppleCare coverage expiration date | AppleCare & Warranty | link | — | — | Click AppleCare & Warranty. | On Mac. |
| MD.09.c146 | Find your AppleCare coverage expiration date | your device | list | — | — | Select your device to see its AppleCare coverage expiration date. | On Mac. |
| MD.09.c147 | Find your AppleCare coverage expiration date | mysupport.apple.com | link | — | — | Go to the site to see details about eligible support, including hardware repairs and technical support. | — |
| MD.09.c148 | Find your AppleCare coverage expiration date | Sign in to your Apple Account | other | — | — | Sign in to see your device's support details. | — |
| MD.09.c149 | Find your AppleCare coverage expiration date | your device | list | — | — | Choose your device to see details about the support you're eligible for. | On mysupport.apple.com, after signing in to your Apple Account. |
| MD.09.c150 | Get the agreement number or proof of coverage | mysupport.apple.com | link | — | — | Go to the site to get the agreement number or proof of coverage. | — |
| MD.09.c151 | Get the agreement number or proof of coverage | Sign in to your Apple Account | other | — | — | Sign in to your Apple Account. | — |
| MD.09.c152 | Get the agreement number or proof of coverage | your device | list | — | — | Choose your device. | After signing in to your Apple Account. |
| MD.09.c153 | Get the agreement number or proof of coverage | View Proof of Coverage | link | — | — | Click to view proof of coverage. | If the proof of coverage is not visible, make sure your Apple Account is set up with two-factor authentication. |

Notes: The page does not state a macOS version. Not all AppleCare products are available in all countries or regions. The page says to learn what to do if you can't find your AppleCare plan, but does not provide those instructions in the supplied text.

#### Independent macOS 27 reports for MD.09

| ID | Kind | Pane | Claim | Source |
|---|---|---|---|---|
| MD.09.x01 | new | General > AutoFill & Passwords | A new switch controls which apps can supply AutoFill data. Passwords is the only option by default and is turned on; third-party password managers may also appear. | T4 [geeksmodo](https://geeksmodo.com/macos-system-settings/) |
| MD.09.x02 | behavior | General > Software Update | macOS 27 Golden Gate is available through System Settings > General > Software Update. | T4 [iGeeksBlog](https://www.igeeksblog.com/macos-27-features/) |
| MD.09.x03 | behavior | System Settings → General → About | The Intel-based Apps details view lists Intel-based apps on the Mac. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/13/) |
| MD.09.x04 | new | System Settings > General > About | A new “Intel-based Apps” section lists recently launched Intel-exclusive apps; universal binaries do not appear in the list. | T4 [Ars Technica](https://arstechnica.com/gadgets/2026/09/macos-27-golden-gate-the-ars-technica-review/) |
| MD.09.x05 | behavior | System Settings > Storage | After a fresh install, the Storage tab reported about 14GB for Apple Intelligence on the M1 MacBook Air and MacBook Neo, and 22.42GB on the M3 MacBook Air. | T4 [Ars Technica](https://arstechnica.com/gadgets/2026/09/macos-27-golden-gate-the-ars-technica-review/) |
| MD.09.x06 | behavior | Storage | To check storage, open System Settings > General > Storage. The page shows space used by Applications, Documents, and Photos, but does not identify Apple Intelligence storage separately. | T4 [PCMag UK](https://uk.pcmag.com/macos/167598/is-apple-intelligence-stealing-your-macs-storage-heres-how-to-turn-it-off) |
| MD.09.x07 | identifier | General | The primary General command opens the section. Listed subsections include About, Software Update, Storage, AirDrop & Handoff, Login Items, AppleCare & Warranty, Language & Region, Date & Time, Sharing, Time Machine, Transfer or Reset, and Startup Disk. The Language & Region and later listed subsections are marked macOS 14 Sonoma; Startup Disk has a legacy alternative. | T2 [GitHub](https://github.com/bvanpeski/SystemPreferences/blob/main/macos_preferencepanes-Ventura.md) |
| MD.09.x08 | behavior | General > Login Items | Login Items are used to choose apps that launch automatically when the Mac starts. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |

## 13.10 MD.10 Accessibility

Status: **PARTIAL**. 25 Accessibility subpages captured; label conflicts with independent 27 reports (GAP-B4-03, 04).


### Accessibility > Zoom: Change Zoom settings for accessibility on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mh40579/27/mac/27) · topic `mh40579` · path: Apple menu > System Settings > Accessibility > Zoom · "Open … for me" link: yes

Use Zoom settings for accessibility to make everything on the screen larger or to enlarge the area where the pointer is located.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.10.c001 | Zoom | Use keyboard shortcuts to zoom | checkbox | — | — | Use the listed shortcuts to zoom in or out. | — |
| MD.10.c002 | Zoom | Use trackpad gesture to zoom | checkbox | — | — | Zoom using the indicated trackpad gestures. | — |
| MD.10.c003 | Zoom | Use scroll gesture with modifier keys to zoom | checkbox | — | — | Press and hold the selected modifier key while scrolling with your trackpad or mouse to zoom in or out. | — |
| MD.10.c004 | Zoom | Modifier key for scroll gesture | popup | Control, Option, or Command | — | Choose the modifier key to press and hold while scrolling to zoom. | — |
| MD.10.c005 | Zoom | Zoom style | popup | Full Screen, Split Screen, Picture-in-Picture | — | Choose to zoom the entire screen, a custom area of the screen, or just the area where the pointer is located. | — |
| MD.10.c006 | Zoom | Advanced | other | — | — | Specify options for the selected zoom style. | — |
| MD.10.c007 | Zoom | Touch Bar zoom | checkbox | — | — | Display a larger version of the Touch Bar on the screen. | If your Mac has a Touch Bar. |

Notes: The page says Accessibility Shortcuts can be used to quickly turn Zoom on or off. The page refers to listed keyboard shortcuts but does not give the shortcut keys in the provided text.

### Accessibility > Descriptions: Change Audio Descriptions settings for accessibility on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl2a14f1c5/27/mac/27) · topic `mchl2a14f1c5` · path: Apple menu · "Open … for me" link: yes

Use Audio Descriptions settings to listen to descriptions of visual content in movies, TV programs, and other media, if available.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.10.c008 | — | Play audio descriptions when available | checkbox | — | — | Automatically play a spoken description of visual content in media. | Audio descriptions must be available in the media. |

Notes: The supplied text does not include the complete navigation path or a macOS version.

### Accessibility > Hover Text: Change Hover Text settings for accessibility on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl46661917/27/mac/27) · topic `mchl46661917` · path: Apple menu > System Settings > Accessibility > Hover Text · "Open … for me" link: no

Turn on and customize Hover Text, Hover Color, and Hover Typing, including their appearance and placement.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.10.c009 | Hover Text | Hover Text | toggle | — | — | View a large, high-resolution version of text you’re reading, as well as other onscreen items like app icons. | — |
| MD.10.c010 | Hover Text | Activation modifier | popup | — | — | Choose the modifier key to press that turns on Hover Text. | — |
| MD.10.c011 | Hover Text | Triple-press modifier to set activation lock | checkbox | — | — | Keep Hover Text active for a period of time—for example, when filling out a form—without needing to press the modifier key every time you move the pointer over text. | — |
| MD.10.c012 | Hover Text | Text size | popup | — | — | Choose the size for text shown in the Hover Text window. | — |
| MD.10.c013 | Hover Text | Text font | popup | — | The font is the same as your system font. | Choose the font for text shown in the Hover Text window. | — |
| MD.10.c014 | Hover Text | Colors | other | — | — | Choose the color for text in the Hover Text window or the colors for the window itself. You can also choose a color to outline the text or item you’re hovering over. | — |
| MD.10.c015 | Hover Color | Hover Color | toggle | — | — | View a large, high-resolution version of colors in text you’re reading, as well as other onscreen items like app icons. | — |
| MD.10.c016 | Hover Color | Activation modifier | popup | — | — | Choose the modifier key to press that turns on Hover Color. | — |
| MD.10.c017 | Hover Color | Text size | popup | — | — | Choose the size for text shown in the Hover Color window. | — |
| MD.10.c018 | Hover Color | Text font | popup | — | The font is the same as your system font. | Choose the font for text shown in the Hover Color window. | — |
| MD.10.c019 | Hover Color | Colors | other | — | — | Choose the color for text in the Hover Color window or the colors for the window itself. You can also choose a color to outline the text or item you’re hovering over. Hover Color shares appearance settings with Hover Text. | — |
| MD.10.c020 | Hover Typing | Hover Typing | toggle | — | — | Display a larger, editable version of text as you type it. | — |
| MD.10.c021 | Hover Typing | Text size | popup | — | — | Choose the size for text shown in the Hover Typing window. | — |
| MD.10.c022 | Hover Typing | Text font | popup | — | The font is the same as your system font. | Choose the font for text shown in the Hover Typing window. | — |
| MD.10.c023 | Hover Typing | Text-entry location | popup | Near the line where you’re entering text; at one of the corners of the screen; Custom. | — | Choose the position of the Hover Typing window. To set your own position, choose Custom, then drag the window to where you want it shown. | — |
| MD.10.c024 | Hover Typing | Colors | other | — | — | Choose the color of the text in the Hover Typing window, the color of the insertion point, or the colors of the Hover Typing window. You can also choose a color to outline the text field you’re hovering over. | — |

Shortcuts stated: Quickly press the modifier key three times to turn the activation lock on or off.

Notes: The page does not state a macOS version. The page says Hover Color shares appearance settings with Hover Text.

### Accessibility > VoiceOver: Change VoiceOver settings for accessibility on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mh40578/27/mac/27) · topic `mh40578` · path: Apple menu · "Open … for me" link: no

Use VoiceOver settings to turn VoiceOver on or off, open the VoiceOver tutorial, or open VoiceOver Utility to customize settings.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.10.c025 | — | VoiceOver | toggle | — | — | Turn VoiceOver on or off. | — |
| MD.10.c026 | — | Open VoiceOver Tutorial | other | — | — | Learn how to use VoiceOver. | — |
| MD.10.c027 | — | Open VoiceOver Utility | other | — | — | Customize VoiceOver settings. | — |

Shortcuts stated: Accessibility Shortcuts

Notes: The page does not state a macOS version. The supplied navigation instruction ends at “Apple menu”; no further path is provided. No default settings, hardware or account conditions, or additional effects are stated.

### Accessibility > Name Recognition: Have your Mac notify you when your name is called

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl36b3b4a1/27/mac/27) · topic `mchl36b3b4a1` · path: Apple menu > System Settings > Accessibility > Hearing > Name Recognition · "Open … for me" link: yes

Name Recognition can continuously listen for your name and notify you when it recognizes it. You can record how you or someone else says your name to help your Mac recognize it.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.10.c028 | Set up Name Recognition | Set Up | button | — | — | Starts the Name Recognition setup instructions. | Available on Mac computers with Apple silicon and only in supported languages. |
| MD.10.c029 | Add more than one name to Name Recognition | Add Name | button | — | — | Starts the instructions for adding another name for your Mac to recognize. | Available on Mac computers with Apple silicon and only in supported languages. |

Notes: The page does not state a macOS version. Don’t rely on your Mac to recognize your name in circumstances where you may be harmed or injured, or in high-risk or emergency situations. The page links to Accessibility features for hearing on Mac and Get captions of spoken and computer audio on Mac.

### Accessibility > Live Speech: Change Live Speech settings for accessibility on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl4876c66a/27/mac/27) · topic `mchl4876c66a` · path: Apple menu > System Settings > Accessibility > Live Speech · "Open … for me" link: no

Use Live Speech settings to turn on Live Speech, choose the language and voice, set the text size, and manage phrases and categories.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.10.c030 | Live Speech settings | Live Speech | toggle | On or off | — | Turn Live Speech on or off to have what you type spoken out loud using synthesized speech. | — |
| MD.10.c031 | Live Speech settings | System speech language | popup | Languages available for Live Speech | — | Choose the language you want to type in and have spoken. | Live Speech is not available in all languages. |
| MD.10.c032 | Live Speech settings | Voice | popup | Provided system voices or a Personal Voice | — | Choose the voice used to speak what you type. | — |
| MD.10.c033 | Live Speech settings | Font size | other | — | — | Set the text size for the Live Speech window. | — |
| MD.10.c034 | Live Speech settings | Categories | list | Recently used phrases and saved phrases | — | View and edit the phrases you’ve used recently and the phrases you’ve saved. | — |
| MD.10.c035 | Live Speech settings | Add Category | button | — | — | Organize your saved phrases into categories, such as for different activities—like work, school, or gaming. | — |

Notes: The page does not specify a macOS version. Live Speech is not available in all languages; the page refers to the macOS Tahoe Feature Availability article. The page says Accessibility Shortcuts can be used to quickly turn Live Speech on or off, but does not provide a specific keyboard shortcut. No defaults, restart requirements, confirmation dialogs, or authentication prompts are stated.

### Accessibility > Spoken Content: Change Read & Speak settings for accessibility on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/spch638/27/mac/27) · topic `spch638` · path: Apple menu > System Settings > Accessibility > Read & Speak · "Open … for me" link: yes

Use Read & Speak settings to customize the system voice, hear text spoken aloud, receive spoken announcements, and set other options for spoken content.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.10.c036 | Read & Speak | Accessibility Reader | other | — | — | Read or listen to text and customize fonts, layout, and background colors. To customize, click “Listen to or change how text appears in apps with Accessibility Reader.” | — |
| MD.10.c037 | Read & Speak | Speak selection | toggle | — | — | Have your Mac speak selected text when you press the specified keyboard shortcut. | — |
| MD.10.c038 | Speak selection | Keyboard shortcut | field | A key combination, such as Option+Tab | — | Press the key combination you want to use. | — |
| MD.10.c039 | Speak selection | Highlight content | popup | Words, sentences, both, or None | — | Choose whether spoken words, sentences, or both are highlighted. Choose None to prevent spoken content from being highlighted. | — |
| MD.10.c040 | Speak selection | Word color | popup | — | — | Choose the color used to highlight words as they’re spoken. | — |
| MD.10.c041 | Speak selection | Sentence color | popup | — | — | Choose the color used to highlight sentences as they’re spoken. | — |
| MD.10.c042 | Speak selection | Sentence style | popup | Underline or “Background color” | — | Choose how to indicate sentences as they’re spoken. | — |
| MD.10.c043 | Speak selection | Show controller | popup | Automatically, Never, or Always | — | Choose when to show the controller. | — |
| MD.10.c044 | Read & Speak | Speak item under the pointer | toggle | — | — | Have your Mac identify items under the pointer. | — |
| MD.10.c045 | Speak item under the pointer | When to speak the item | popup | All the time or only when you use zoom | — | Choose when your Mac speaks the item. | — |
| MD.10.c046 | Speak item under the pointer | Verbosity | popup | High, Medium, or Low | — | Choose the level of detail. | — |
| MD.10.c047 | Speak item under the pointer | Wait before speaking | other | — | — | Choose how long your Mac waits before speaking. | — |
| MD.10.c048 | Read & Speak | Speak announcements | toggle | — | — | Have your Mac announce and speak dialogs and other announcements. | — |
| MD.10.c049 | Speak announcements | Voice | popup | — | — | Select a voice for announcements. | — |
| MD.10.c050 | Speak announcements | Phrase | popup | Examples include the app name or “Attention!” | — | Choose the phrase to hear before an announcement is spoken. | — |
| MD.10.c051 | Speak announcements | Wait before speaking | other | — | — | Choose how long your Mac waits before speaking. | — |
| MD.10.c052 | Speak announcements | Play Sample | button | — | — | Hear a sample. | — |
| MD.10.c053 | Read & Speak | Speak typing feedback | toggle | — | — | Have your Mac speak what you type or the keys you press. | — |
| MD.10.c054 | Speak typing feedback | Echo characters | checkbox | — | — | Your Mac speaks each character you type. | — |
| MD.10.c055 | Speak typing feedback | Echo words | checkbox | — | — | Your Mac speaks each word you type. | — |
| MD.10.c056 | Speak typing feedback | Echo selection changes | checkbox | — | — | When you use the arrow keys to move the insertion point forward or back, your Mac speaks the character to the right or left of the insertion point. | — |
| MD.10.c057 | Speak typing feedback | Echo modifier keys | checkbox | — | — | Your Mac speaks each modifier key you press, including the Caps Lock key. | — |
| MD.10.c058 | Read & Speak | System speech language | popup | — | — | Choose the language you want your Mac to use to speak text and alerts, if different from the system language. | Applies when choosing a speech language different from the system language. |
| MD.10.c059 | Read & Speak | System voice | popup | — | — | Choose a voice to speak text and alerts. Click “Change the voice your Mac uses to speak text.” | Voices are not available in all languages. |
| MD.10.c060 | Read & Speak | Detect languages | toggle | — | — | If you read text in multiple languages, have your Mac automatically switch between them when it speaks text. When this option is off, text is spoken using the system speech language you set above. | Relevant when reading text in multiple languages. |
| MD.10.c061 | Read & Speak | Pronunciations | toggle | — | — | If you’ve defined custom pronunciations for Read & Speak, turn them on or off. To edit custom pronunciations, click the edit control. | Custom pronunciations must have been defined to turn them on or off. |
| MD.10.c062 | Pronunciations | Edit custom pronunciations | button | — | — | Open the controls for editing custom pronunciations. | — |

Shortcuts stated: Option+Tab is given as an example keyboard shortcut for Speak selection.; Command-F5 turns VoiceOver off.; If your Mac or Magic Keyboard has Touch ID, press and hold the Command key while you quickly press Touch ID three times to turn VoiceOver off.

Notes: The page does not state a macOS version that it applies to. It refers to the Apple Support article “macOS Tahoe Feature Availability” when noting that voices are not available in all languages. You can set keyboard shortcuts to quickly turn on or off options like “Speak selection,” “Speak item under the pointer,” and “Speak typing feedback” using Apple menu > System Settings > Keyboard > Keyboard Shortcuts. If your Mac speaks unexpectedly, VoiceOver may have been turned on inadvertently.

### Accessibility > Vocal Shortcuts: Change Vocal Shortcuts settings for accessibility on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl0b0b8c01/27/mac/27) · topic `mchl0b0b8c01` · path: Apple menu > System Settings > Accessibility > Vocal Shortcuts · "Open … for me" link: yes

Use Vocal Shortcuts settings to set up shortcuts, turn Vocal Shortcuts on or off, and manage the shortcuts you’ve created.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.10.c063 | — | Vocal Shortcuts | toggle | — | — | Set up Vocal Shortcuts, or turn it on or off. See Set up Vocal Shortcuts. | Available only on Mac computers with Apple silicon. |
| MD.10.c064 | — | List of shortcuts | list | — | — | The list of Vocal Shortcuts you’ve created. See Add a Vocal Shortcut. | — |

Notes: The page does not state a macOS version.

### Accessibility > Audio: Change Audio settings for accessibility on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/unac611/27/mac/27) · topic `unac611` · path: Apple menu > System Settings > Accessibility > Audio · "Open … for me" link: no

Customize accessibility audio settings, including stereo playback, Spatial Audio, background sounds, headphone controls, and Headphone Accommodations.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.10.c065 | General | Flash the screen when an alert sound occurs | toggle | — | — | Make your screen flash when you receive an alert. | — |
| MD.10.c066 | General | Play stereo audio as mono | toggle | — | — | Make your Mac play stereo audio tracks as mono audio in the left and right speakers or headphones. | — |
| MD.10.c067 | General | Spatial audio follows head movements | toggle | — | On | When you watch or listen to content that supports Spatial Audio on your Mac, the audio automatically adjusts as you move your head. Turning this setting off keeps Spatial Audio on, but the apparent location of sounds doesn’t change as you move your head. | Available only on Mac computers with Apple silicon when compatible headphones are paired with your Mac. Spatial Audio is available with AirPods 3 or later, AirPods Pro, and AirPods Max. |
| MD.10.c068 | Your AirPods or other compatible headphones | Press speed | other | — | — | When an action requires pressing the buttons, force sensor, or Touch control on your headphones multiple times, adjust how quickly you must press before the action occurs. | Available options vary based on the model of headphones paired with your Mac. |
| MD.10.c069 | Your AirPods or other compatible headphones | Press and hold duration | other | — | — | Adjust the duration required to press and hold the buttons, force sensor, or Touch control on your headphones before an action occurs. | Available options vary based on the model of headphones paired with your Mac. |
| MD.10.c070 | Your AirPods or other compatible headphones | Noise cancellation with one earbud | toggle | — | — | Allow noise cancellation mode when you’re only wearing one earbud. | Available options vary based on the model of headphones paired with your Mac. |
| MD.10.c071 | Your AirPods or other compatible headphones | Tone volume | slider | — | — | Adjust the volume of the sound effects, such as low battery alerts, played by your headphones. | Available options vary based on the model of headphones paired with your Mac. |
| MD.10.c072 | Background Sounds | Background sounds | toggle | — | — | Turn background sounds on or off. | — |
| MD.10.c073 | Background Sounds | Background sound | other | Background sounds such as Rain or Balanced Noise. | — | Choose which background sound to play. | — |
| MD.10.c074 | Background Sounds | Background sounds volume | slider | — | — | Adjust the volume of the background sound. | — |
| MD.10.c075 | Background Sounds | Equalizer | other | — | — | Adjust the volume of different frequency ranges in background sounds. | — |
| MD.10.c076 | Background Sounds | Timer | other | — | — | Set a timer to stop playing background sounds at a specific time or after an amount of time. | — |
| MD.10.c077 | Background Sounds | Turn off background sounds when your Mac is not in use | toggle | — | — | Automatically turn off background sounds when your Mac is in sleep, the screen saver is on, or the screen is locked. | — |
| MD.10.c078 | Background Sounds | Sound Settings | link | — | — | Adjust the system volume, select a different sound input or output device, and more. See Change Sound settings. | — |
| MD.10.c079 | Headphone Accommodations | Headphone Accommodations | toggle | — | — | Use Headphone Accommodations to amplify soft sounds and adjust certain frequencies to best suit your hearing. | Headphone Accommodations on Mac works with AirPods Pro 2 and AirPods Pro 3. |
| MD.10.c080 | Headphone Accommodations | Custom Audio Setup | button | — | — | With some compatible headphones, personalize your headphone audio based on what sounds good to you. Click Custom Audio Setup, then follow the onscreen instructions. | Requires compatible headphones. |
| MD.10.c081 | Headphone Accommodations | Tune audio for | popup | Balance Tone; Vocal Range; Brightness | — | Choose an option to boost different frequencies. Balance Tone boosts a range of frequencies; Vocal Range boosts middle frequencies; Brightness boosts high frequencies. | — |
| MD.10.c082 | Headphone Accommodations | Optimization | slider | — | — | Drag the slider to set an amplification level for soft sounds. | — |
| MD.10.c083 | Headphone Accommodations | Apply with | other | Phone; Media | — | Phone applies Headphone Accommodations settings to phone calls. Media applies them to media playback like music, movies, and podcasts. | — |

Notes: Available options vary based on the model of headphones paired with your Mac. The page does not state which macOS version it applies to.

### Accessibility > Motion: Change Motion settings for accessibility on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchla3c4f1da/27/mac/27) · topic `mchla3c4f1da` · path: Apple menu > System Settings > Accessibility > Motion · "Open … for me" link: yes

Use Motion settings to customize onscreen movement, dim flashing lights in compatible media, pause rapid animated images, stop the cursor from blinking, and use Vehicle Motion Cues.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.10.c084 | Motion | Reduce motion | toggle | — | — | Customize the movement of elements on the screen—for example, when you open apps, switch between desktops, or open and close Notification Center. | — |
| MD.10.c085 | Motion | Dim flashing lights | toggle | — | — | For compatible media, automatically dim the display of content that depicts flashing or strobing lights. | Available only on Mac computers with Apple silicon.; Content is processed on device in real time. Should not be relied upon for the treatment of any medical condition. |
| MD.10.c086 | Motion | Auto-play animated images | toggle | — | — | Turn off to automatically pause the display of rapid animated images, such as GIFs in Messages and Safari. | — |
| MD.10.c087 | Motion | Prefer non-blinking cursor | toggle | — | — | Stop the blinking of the vertical bar that appears at the insertion point where you’re typing. | — |
| MD.10.c088 | Motion | Vehicle Motion Cues | toggle | — | — | May help reduce vehicle motion sickness while using a Mac and riding as a passenger in a car or other on-road vehicle. | Available on Mac laptop computers. Not available on MacBook Neo, MacBook Air (M1), or 13-inch MacBook Pro (M1) or earlier.; Should not be used while operating a moving vehicle or in other situations requiring attention to safety. Works best when seated facing forward in the vehicle. |
| MD.10.c089 | Vehicle Motion Cues | Customize Appearance | button | — | — | Opens options for customizing Vehicle Motion Cues. | — |
| MD.10.c090 | Customize Appearance | Pattern | popup | Regular; Dynamic | — | Select Regular for a stable and predictable pattern of onscreen dots, or Dynamic for a more engaging visual experience. | — |
| MD.10.c091 | Customize Appearance | Color | popup | A color for the onscreen dots. | — | Select a color for the onscreen dots. Color saturation adjusts automatically to maintain contrast with the content behind each dot. | — |
| MD.10.c092 | Customize Appearance | Large dots | toggle | — | — | Turn on to increase the size of the dots that appear onscreen. | — |
| MD.10.c093 | Customize Appearance | More dots | toggle | — | — | Turn on to increase the number of dots that appear onscreen. | — |

Notes: Using a Mac in some circumstances may distract you and might cause a dangerous situation. Observe rules that prohibit or restrict the use of devices or headphones. The page does not state a macOS version or any default settings.

### Accessibility > Personal Voice: Change Personal Voice settings for accessibility on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl4b5f02ec/27/mac/27) · topic `mchl4b5f02ec` · path: Apple menu > System Settings > Accessibility > Personal Voice · "Open … for me" link: yes

Use Personal Voice settings to view or create synthesized voices that sound like you, share them across devices, and manage which apps can use them.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.10.c094 | — | Personal Voice | list | — | — | View a list of your Personal Voices, or record a new one. | Available only on Mac computers with Apple silicon; not available in all languages. |
| MD.10.c095 | Personal Voice | Create a Personal Voice | button | — | — | Click to record a new Personal Voice. | — |
| MD.10.c096 | — | Share across devices | other | — | — | Use your Personal Voices on all devices where you’re signed in to the same Apple Account. | Requires signing in to the same Apple Account on the devices. |
| MD.10.c097 | — | Allow applications to use your Personal Voice | list | — | — | View a list of apps that you’ve given permission to use your Personal Voice. You can add more apps or revoke an app’s access. | Personal Voice can be used with Live Speech and third-party apps that you allow, such as augmentative and alternative communication (AAC) apps. |

Notes: Personal Voice can be used only to create a voice that sounds like you on device, using your own voice, and for your own personal, noncommercial use. The page does not state a macOS version, defaults, or any restart, logout, confirmation, or authentication effects.

### Accessibility > Live Captions: Change Live Captions settings for accessibility on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchla0b36db8/27/mac/27) · topic `mchla0b36db8` · path: Apple menu > System Settings > Accessibility > Live Captions · "Open … for me" link: yes

Use Live Captions settings to turn Live Captions on or off, adjust caption appearance and language, and choose whether captions are limited to FaceTime.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.10.c098 | Live Captions | Live Captions | toggle | — | — | Turn Live Captions on or off. | Available only on Mac computers with Apple silicon; not available in all languages, countries, or regions. |
| MD.10.c099 | Live Captions | Font family | other | — | — | Choose a font for Live Captions. | — |
| MD.10.c100 | Live Captions | Font size | other | — | — | Choose a text size for Live Captions. | — |
| MD.10.c101 | Live Captions | Font color | other | — | — | Choose a font color for Live Captions. | — |
| MD.10.c102 | Live Captions | Background color | other | — | — | Choose a background color for the Live Captions window. | — |
| MD.10.c103 | Live Captions | Language | other | — | — | Choose a language for Live Captions. | Live Captions is not available in all languages. |
| MD.10.c104 | Live Captions | Live Captions in FaceTime | toggle | — | — | Turn this on to use Live Captions only in FaceTime. By default, Live Captions transcribes audio from all apps. | Live Captions is available only on Mac computers with Apple silicon. |

Notes: The page warns that Live Captions is not available in all languages, countries, or regions. The accuracy of Live Captions may vary and should not be relied upon in high-risk or emergency situations.

### Accessibility > Siri: Change Siri settings for accessibility on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl5715a9f2/27/mac/27) · topic `mchl5715a9f2` · path: Apple menu · "Open … for me" link: yes

Use Siri settings for accessibility to speak responses or have Siri listen for atypical speech patterns.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.10.c105 | — | Listen for atypical speech | toggle | — | — | Have Siri recognize a wider range of speech patterns. This may be helpful if you have an acquired or progressive condition that affects speech, such as cerebral palsy, amyotrophic lateral sclerosis (ALS), or a stroke. | Only available in U.S. English. |
| MD.10.c106 | — | Spoken responses | other | — | — | Choose whether Siri is always silent or speaks responses in certain instances, like when you’re driving or wearing headphones. | — |
| MD.10.c107 | — | Siri Settings | link | — | — | More options, such as setting a keyboard shortcut to activate Siri and showing what Siri says onscreen, are available in Siri settings. | — |

Notes: The page mentions setting a keyboard shortcut to activate Siri but does not specify a shortcut. The page suggests using Vocal Shortcuts to teach the Mac to perform an action when you speak a chosen word or sound if you have moderate to severe atypical speech. The navigation text provided ends at “Apple menu”; no further path is shown.

### Accessibility > Captions: Change Captions settings for accessibility on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mh43180/27/mac/27) · topic `mh43180` · path: Apple menu > System Settings > Accessibility > Captions · "Open … for me" link: no

Use Captions settings to control subtitle styles and whether closed captions are preferred when available.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.10.c108 | Captions | List of caption styles | list | Available built-in styles and styles that you added. | — | Shows available built-in caption styles and styles that you added. | — |
| MD.10.c109 | Captions | Add | button | — | — | Create a style. | — |
| MD.10.c110 | Captions | Remove | button | — | — | Delete a style that you added. | — |
| MD.10.c111 | Captions | Prefer closed captions and SDH | checkbox | — | — | If they’re available, use closed captions or subtitles for the deaf and hard of hearing (SDH) instead of standard subtitles for content played on your Mac. | Closed captions or SDH must be available. |

### Accessibility > Head Pointer: Move the pointer using head pointer on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlb2d4782b/27/mac/27) · topic `mchlb2d4782b` · path: Apple menu > System Settings > Accessibility > Motor > Pointer Control · "Open … for me" link: yes

Use the camera to move the pointer by moving your face or head. The page explains how to enable and customize head pointer, use it, troubleshoot it, and recalibrate it.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.10.c112 | Turn on and customize head pointer | Head pointer | toggle | — | — | Turns head pointer on or off. | — |
| MD.10.c113 | Turn on and customize head pointer | Head pointer options | button | — | — | Opens options for how the pointer moves, pointer speed, switches for head pointer actions, and which camera to use. The button’s exact label is not provided. | — |
| MD.10.c114 | Turn on and customize head pointer | How the pointer moves | other | — | — | An option that specifies how the pointer moves. The available choices and exact control label are not provided. | — |
| MD.10.c115 | Turn on and customize head pointer | Pointer speed | other | — | — | An option that controls how far the pointer moves in response to face or head movement. | — |
| MD.10.c116 | Turn on and customize head pointer | Switches for head pointer actions | other | Examples mentioned include keyboard keys, assistive switches, and facial expressions. | — | Options to specify switches for head pointer actions. The exact control labels and choices are not provided. | — |
| MD.10.c117 | Turn on and customize head pointer | Camera | other | — | — | An option to choose which camera head pointer uses. | The selected camera must be on, facing the user, and unobstructed. |
| MD.10.c118 | Recalibrate head pointer | Head pointer | toggle | — | — | Turn on head pointer before opening its options to enable recalibration. | — |
| MD.10.c119 | Recalibrate head pointer | Use a switch or facial expression to recalibrate | toggle | — | — | Enables recalibration using a switch or facial expression. After activation, adjust your face or head position and use the switch to start recalibration; an onscreen countdown runs from 3. | Repositions the pointer in the middle of the screen and resets the default face or head position, depending on the movement option selected. |

Notes: The page does not state a macOS version. The page mentions Accessibility Shortcuts as a way to quickly turn head pointer on or off, but gives no specific keyboard shortcut. The example key “z” is a user-configured switch for pausing head pointer, not a stated keyboard shortcut.

### Accessibility > Display: Change Display settings for accessibility on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/unac089/27/mac/27) · topic `unac089` · path: Apple menu > System Settings > Accessibility > Display · "Open … for me" link: yes

Adjust display appearance, text size, pointer visibility, and color filters to make items on the screen easier to see.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.10.c120 | Display | Invert colors | toggle | — | — | Invert colors on your display—for example, show white text on a black background. | — |
| MD.10.c121 | Display | Invert colors mode | popup | Classic; Smart | — | Choose to invert colors everywhere (Classic), or everywhere except images and video (Smart). | — |
| MD.10.c122 | Display | Increase contrast | toggle | — | — | Increase the contrast of items on the screen (such as borders around buttons or boxes) without changing the contrast of the screen itself. | — |
| MD.10.c123 | Display | Reduce transparency | toggle | — | — | Replace the transparent effect used on some backgrounds in macOS with a solid background, to improve contrast and readability. | — |
| MD.10.c124 | Display | Differentiate without color | toggle | — | — | Use shapes, in addition to or instead of color, to convey status or information. | — |
| MD.10.c125 | Display | Show window title icons | toggle | — | — | Show a window’s icon (if available) in its title bar. The icon can make it easier to distinguish among windows or tabs when you have several open—for example, Finder windows showing Recents, Downloads, and iCloud Drive. | — |
| MD.10.c126 | Display | Show toolbar button shapes | toggle | — | — | Show a subtle border around toolbar buttons to indicate the area to click when you select a toolbar button. | — |
| MD.10.c127 | Display | Display contrast | slider | — | — | Drag the slider to increase or decrease the screen contrast. | — |
| MD.10.c128 | Text | Text size | other | — | — | Set your preferred reading size for compatible apps and system features. | — |
| MD.10.c129 | Text | Menu bar size | popup | Large; Default | — | Set the font size in the menu bar to Large, or back to Default. | You must log out and log in again to see the change in menu bar font size. |
| MD.10.c130 | Text | Prefer horizontal text | toggle | — | — | Prefer horizontal text in languages that prefer vertical text. | — |
| MD.10.c131 | Pointer | Shake mouse pointer to locate | toggle | — | — | Quickly move the mouse, or your finger on the trackpad, to temporarily enlarge the pointer so you can find it. | — |
| MD.10.c132 | Pointer | Pointer size | slider | — | — | Increase or decrease the size of the pointer. | — |
| MD.10.c133 | Pointer | Pointer outline color | other | — | — | Select a color for the outline of the pointer. | — |
| MD.10.c134 | Pointer | Pointer fill color | other | — | — | Select a color to fill the inside of the pointer outline. | — |
| MD.10.c135 | Pointer | Reset Colors | button | — | — | Reset the pointer outline and fill colors to the default settings. | Resets the outline to white and the fill to black. |
| MD.10.c136 | Color Filters | Color filters | toggle | — | — | Adjust display colors using the selected filter type. | — |
| MD.10.c137 | Color Filters | Filter type | popup | Grayscale; Red/Green (for protanopia); Green/Red (for deuteranopia); Blue/Yellow (for tritanopia); Color Tint | — | Choose a filter type. | — |
| MD.10.c138 | Color Filters | Intensity | slider | — | — | Drag the slider to adjust the intensity of the filter. | — |
| MD.10.c139 | Color Filters | Tint | other | — | — | Click the Tint color well to customize the color. | Available if you choose the Color Tint filter. |

Notes: The page suggests using Accessibility Shortcuts to quickly turn on or off Invert Colors, Increase Contrast, Reduce Transparency, and Color Filters; it does not specify a keyboard shortcut.

### Accessibility: Change Accessibility settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp1400/27/mac/27) · topic `mchlp1400` · path: Apple menu > System Settings > Accessibility · "Open … for me" link: yes

The Accessibility settings pane lets you customize Mac features for vision, hearing, mobility, and speech. The page describes categories that open more detailed settings.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.10.c140 | Accessibility | Open Accessibility settings for me | link | — | — | Opens Accessibility settings. | — |
| MD.10.c141 | Accessibility | VoiceOver | link | — | — | Open VoiceOver settings to turn the built-in screen reader on or off, open VoiceOver Utility, or start interactive VoiceOver training. | — |
| MD.10.c142 | Accessibility | Zoom | link | — | — | Open Zoom settings to enlarge the entire screen or an area of it, use Hover Text, or set zoom separately for a second display. | If you’re using a second display, set the zoom for it separately. |
| MD.10.c143 | Accessibility | Hover Text | link | — | — | Open Hover Text settings to view a larger version of text under the pointer, or display a larger, editable version of text as you type. | — |
| MD.10.c144 | Accessibility | Display | link | — | — | Open Display settings to make onscreen items, including the pointer, easier to see; make text bigger; reduce onscreen motion; and apply color filters or tints. | — |
| MD.10.c145 | Accessibility | Read & Speak | link | — | — | Open Read & Speak settings to customize the system voice; use Accessibility Reader; hear when alerts or apps need your attention; speak selected text when you press a designated key; or speak items under the pointer or text you type. | — |
| MD.10.c146 | Accessibility | Audio Descriptions | link | — | — | Open Descriptions settings to hear descriptions of visual content in movies, TV programs, and other media. | Descriptions must be available for the media. |
| MD.10.c147 | Accessibility | Hearing Devices | link | — | — | Open Hearing Devices settings to pair Made for iPhone (MFi) hearing devices and adjust their settings. | MFi hearing devices can be paired only with select Mac computers with the M1 chip, and all Mac computers with the M2 chip or later. |
| MD.10.c148 | Accessibility | Audio | link | — | — | Open Audio settings to play stereo sound as mono, turn Spatial Audio on or off, boost certain frequencies or soft sounds, and set other accessibility options for headphones. | — |
| MD.10.c149 | Accessibility | RTT | link | — | — | Open RTT settings to set up the Mac to make and receive Real-Time Text (RTT) calls. | Shown only when the iPhone and Mac are set up for Wi-Fi calling and RTT phone calls are enabled on the iPhone. Wi-Fi calling and RTT are not available in all countries or regions, or supported by all carriers. |
| MD.10.c150 | Accessibility | Captions | link | — | — | Open Captions settings to control subtitle styling and whether to use closed captions and SDH instead. | Closed captions and SDH must be available. |
| MD.10.c151 | Accessibility | Live Captions | link | — | — | Open Live Captions settings to turn Live Captions on or off and control caption styling. | Available on Mac computers with Apple silicon; not available in all languages, countries, or regions. |
| MD.10.c152 | Accessibility | Name Recognition | link | — | — | Open Name Recognition settings to have the Mac continuously listen for your name and notify you when it recognizes it. | Available on Mac computers with Apple silicon and only in English. |
| MD.10.c153 | Accessibility | Voice Control | link | — | — | Open Voice Control settings to control the Mac and apps with standard voice commands or commands you create. | — |
| MD.10.c154 | Accessibility | Keyboard | link | — | — | Open Keyboard settings for accessibility to use Full Keyboard Access, Sticky Keys, Slow Keys, the Accessibility Keyboard, or Dwell. | Dwell is described for use with eye- or head-tracking technology. |
| MD.10.c155 | Accessibility | Pointer Control | link | — | — | Open Pointer Control settings to make a mouse or trackpad easier to use; move the pointer using a keyboard or numeric keypad; use keyboard keys, assistive switches, or facial expressions for mouse actions; or move the pointer using head tracking. | — |
| MD.10.c156 | Accessibility | Switch Control | link | — | — | Open Switch Control settings to turn on and set options for using adaptive accessories to navigate and interact with the Mac. | — |
| MD.10.c157 | Accessibility | Live Speech | link | — | — | Open Live Speech settings to have typed text spoken aloud using synthesized speech, add frequently used phrases, and more. | — |
| MD.10.c158 | Accessibility | Personal Voice | link | — | — | Open Personal Voice settings to create a synthesized voice that sounds like you and use it with Live Speech, in apps, and across your devices. | Available only on Mac computers with Apple silicon and not in all languages. Can be used only with Live Speech and third-party apps you allow, such as AAC apps. The voice can be created only to sound like you, on device, using your own voice, for your own personal, noncommercial use. |
| MD.10.c159 | Accessibility | Vocal Shortcuts | link | — | — | Open Vocal Shortcuts settings to teach the Mac to perform an action when you speak a chosen word or other sound. | — |
| MD.10.c160 | Accessibility | Siri | link | — | — | Open Siri settings for accessibility to have Siri listen for atypical speech. | “Listen for atypical speech” is available only in U.S. English. |
| MD.10.c161 | Accessibility | Shortcut | link | — | — | Open Shortcut settings for accessibility to select items to include in the Accessibility Shortcuts panel. | — |

Notes: The page does not state a macOS version. The page says other System Settings, such as Displays, Sound, Keyboard, Trackpad, and Mouse, offer additional helpful options. Live Captions accuracy may vary and should not be relied upon in high-risk or emergency situations. Do not rely on Name Recognition when failure to recognize your name could result in harm or injury, or in high-risk or emergency situations. The page mentions no defaults, restart requirements, confirmation dialogs, or authentication prompts.

### Accessibility > Hearing Devices: Change Hearing Devices settings for accessibility on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl096a5035/27/mac/27) · topic `mchl096a5035` · path: Apple menu > System Settings > Accessibility > Hearing Devices · "Open … for me" link: yes

Use Hearing Devices settings to pair Made for iPhone (MFi) hearing aids or sound processors with your Mac and adjust their settings.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.10.c162 | Hearing Devices | Available Devices | list | — | — | Lists unpaired hearing devices when they are in range of your Mac. Move the pointer over a device and click Connect to pair it with your Mac. | Made for iPhone hearing devices can be paired only with select Mac computers with the M1 chip, and all Mac computers with the M2 chip or later. The devices must be unpaired and in range.; Connecting pairs the hearing device with your Mac. |
| MD.10.c163 | Hearing Devices | Connect | button | — | — | Connects an available hearing device to your Mac. | Appears for an unpaired hearing device in range.; Pairs the hearing device with your Mac. |
| MD.10.c164 | Hearing Devices | Stream to left hearing device | other | — | — | Sends audio from your Mac to the left hearing device. | Requires a compatible hearing device. |
| MD.10.c165 | Hearing Devices | Stream to right hearing device | other | — | — | Sends audio from your Mac to the right hearing device. | Requires a compatible hearing device. |
| MD.10.c166 | Hearing Devices | Adjust independently | toggle | — | — | Adjust settings such as microphone level and presets separately for the left and right hearing devices. Turn this option off to adjust both devices at the same time. | Depending on the hearing device model, some settings can be adjusted only separately or only for both devices at the same time. |
| MD.10.c167 | Hearing Devices | Control Nearby Devices | other | — | — | Lets you use Hearing Devices settings on your Mac to adjust settings for another device, such as an iPhone. | The hearing devices must be paired with and connected to another device where you’re signed in to your Apple Account. |
| MD.10.c168 | Hearing Devices | Main Volume | other | — | — | Adjusts the overall volume of the hearing devices, including audio streamed from your Mac and sound picked up by the microphone from the surrounding environment. | Available options may vary depending on the hearing device model. |
| MD.10.c169 | Hearing Devices | Microphone Level | other | — | — | Adjusts the volume of sound picked up by the hearing devices from the surrounding environment. | Available options may vary depending on the hearing device model. |
| MD.10.c170 | Hearing Devices | Hearing Range | other | — | — | Adjusts the sensitivity of the hearing device microphones. A higher range picks up sound from farther away; a lower range picks up sound only from nearby. | Available options may vary depending on the hearing device model. |
| MD.10.c171 | Hearing Devices | Tone | other | — | — | Adjusts the amount of treble (high-frequency sound) and bass (low-frequency sound) picked up from the surrounding environment. | Available options may vary depending on the hearing device model. |
| MD.10.c172 | Hearing Devices | Presets | other | Preprogrammed environmental presets; options vary based on how the hearing devices have been set up. | — | Chooses preprogrammed environmental presets for sound picked up from the surrounding environment. If unsure about the options, check with the audiologist who helped set up the devices. | Preset options vary based on how the hearing devices have been set up. |
| MD.10.c173 | Hearing Devices | Streaming Presets | other | Preprogrammed environmental presets; options vary by accessory. | — | Chooses preprogrammed environmental presets for audio streamed using hearing device accessories. | Preset options vary by accessory. |
| MD.10.c174 | Hearing Devices | Forget this device | other | — | — | Unpairs and disconnects the paired hearing devices from your Mac. | Unpairs and disconnects the paired hearing devices from your Mac. |

Notes: Made for iPhone hearing devices can be paired only with select Mac computers with the M1 chip, and all Mac computers with the M2 chip or later. Options may vary depending on the model of the hearing devices. The page does not state a macOS version.

### Accessibility > Voice Control: Change Voice Control settings for accessibility on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/spc002/27/mac/27) · topic `spc002` · path: Apple menu > System Settings > Accessibility > Voice Control · "Open … for me" link: no

Use Voice Control settings to turn Voice Control on or off, customize settings and commands, and create a custom vocabulary.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.10.c175 | Voice Control | Voice Control | toggle | — | — | Turn Voice Control on or off. | — |
| MD.10.c176 | Voice Control | Language | popup | — | — | Choose the language to use when you speak commands to your Mac. | — |
| MD.10.c177 | Voice Control | Microphone | popup | Automatic | — | Choose the microphone to use to speak commands to your Mac. If you choose Automatic from the pop-up menu, your Mac listens to the device you’re most likely to use for speaking commands. | — |
| MD.10.c178 | Voice Control | Show hints | toggle | — | — | Display suggested commands and hints on the screen. | — |
| MD.10.c179 | Voice Control | Play sound when command is recognized | toggle | — | — | Have your Mac play a sound to indicate that it recognized the command you spoke. | — |
| MD.10.c180 | Voice Control | Overlay | popup | — | — | Continuously show item names, numbers, or a numbered grid on the screen. | — |
| MD.10.c181 | Voice Control | Fade overlay after inactivity | other | — | — | When item names, numbers, or a numbered grid are shown, dim the overlay after the specified period of inactivity by the specified amount. | — |
| MD.10.c182 | Voice Control | Voice Control Tutorial | button | — | — | Open an interactive tutorial where you can learn and practice essential Voice Control commands. | — |
| MD.10.c183 | Voice Control | Improve assistive voice features | toggle | — | — | Turn on this option to help Apple improve assistive voice features by sharing audio input and transcripts of your Vocal Shortcuts and Voice Control interactions. Voice Control and Vocal Shortcuts audio input and transcripts are submitted anonymously to Apple and only with your consent. | — |
| MD.10.c184 | Voice Control | Commands | button | — | — | Show the list of commands you can use to control your Mac and use apps. | — |
| MD.10.c185 | Voice Control | Vocabulary | button | — | — | Specify custom words or phrases to use when you speak to your Mac. | Not available for all Voice Control languages. |
| MD.10.c186 | Voice Control | Advanced Actions menu | button | — | — | Show the menu you can use to import, export, or delete custom commands and vocabulary. | — |
| MD.10.c187 | Commands | Commands list | list | Standard commands and commands you created; categories include Basic Navigation and Text Editing. | — | Use the search field to find a command, or scroll through the categories of standard commands. | — |
| MD.10.c188 | Commands | Search field | field | — | — | Find a command. | — |
| MD.10.c189 | Commands | Preview area | info | — | — | Click a command in the list to show information about how to use it. | — |
| MD.10.c190 | Commands | Add | button | — | — | Create your own commands. | — |
| MD.10.c191 | Commands | Remove | button | — | — | Delete commands you created. | — |
| MD.10.c192 | Vocabulary | Vocabulary list | list | — | — | Words or phrases that you added. | — |
| MD.10.c193 | Vocabulary | Add | button | — | — | Add words or phrases to your custom vocabulary. | — |
| MD.10.c194 | Vocabulary | Remove | button | — | — | Remove words or phrases from your custom vocabulary. | — |

### Accessibility > RTT: Change RTT settings for accessibility on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlf4b091e8/27/mac/27) · topic `mchlf4b091e8` · path: Apple menu > System Settings > Accessibility > RTT · "Open … for me" link: yes

Use RTT settings to set up your Mac to make and receive Real-Time Text (RTT) calls.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.10.c195 | RTT settings | Enable RTT | toggle | When enabled, a Mac phone call can use audio, RTT, or RTT relay. | — | Make and receive RTT calls on your Mac without using a special device. | The settings are shown only when your iPhone and Mac are set up for Wi-Fi calling and RTT phone calls are enabled on your iPhone. Wi-Fi calling and RTT are not available in all countries or regions, or supported by all carriers. |
| MD.10.c196 | RTT settings | Send immediately | toggle | On: send characters as you type them. Off: type full messages before sending them. | — | Choose whether characters are sent as you type or whether you type full messages before sending. | The settings are shown only when your iPhone and Mac are set up for Wi-Fi calling and RTT phone calls are enabled on your iPhone. Wi-Fi calling and RTT are not available in all countries or regions, or supported by all carriers. |
| MD.10.c197 | RTT settings | RTT relay number | field | — | — | Set the relay service number to use when you make an RTT relay call on your Mac. If no number is specified, your carrier’s default relay service is used. | The settings are shown only when your iPhone and Mac are set up for Wi-Fi calling and RTT phone calls are enabled on your iPhone. Wi-Fi calling and RTT are not available in all countries or regions, or supported by all carriers. |

Notes: Depending on your network environment, an RTT call may be downgraded to a Teletypewriter (TTY) call. For emergency calls in the U.S. on a Mac, special characters or tones are sent to alert the operator. The operator’s ability to receive or respond to these tones can vary by location; Apple doesn’t guarantee the operator will be able to receive or respond to an RTT call.

### Accessibility > Shortcut: Change Shortcut settings for accessibility on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchla7804b65/27/mac/27) · topic `mchla7804b65` · path: Apple menu > System Settings > Accessibility > Shortcut · "Open … for me" link: yes

Change which accessibility features appear in Accessibility Shortcuts, a panel for quickly turning those features on or off.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.10.c198 | — | List of options | list | — | — | Options you can include in the Accessibility Shortcuts panel. See “Add or remove features from Accessibility Shortcuts.” | — |

Notes: The page does not specify a macOS version.

### Accessibility > Background Sounds: Play background sounds on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl3061cdc6/27/mac/27) · topic `mchl3061cdc6` · path: Apple menu > System Settings > Accessibility > Audio · "Open … for me" link: yes

Turn on ambient background sounds and adjust the sound, equalizer, timer, and when playback stops.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.10.c199 | Background Sounds | Background sounds | toggle | — | — | Turns background sounds on or off. | — |
| MD.10.c200 | Choose a sound | Choose | button | Sound names are not listed on the page. | — | Opens the sound choices; click a sound name to use it as the background sound. | — |
| MD.10.c201 | Choose a sound | Remove downloaded sound | button | — | — | Click the remove control to remove a downloaded sound. The page does not provide its label. | Applies to a downloaded sound. |
| MD.10.c202 | Adjust the volume | Volume | slider | — | — | Drag the slider to increase or decrease the volume of the background sound. | — |
| MD.10.c203 | Adjust the frequency ranges | Choose | button | — | — | Opens the Equalizer settings. | — |
| MD.10.c204 | Adjust the frequency ranges | Equalizer | toggle | — | — | Turns on the Equalizer. | — |
| MD.10.c205 | Adjust the frequency ranges | Equalizer grid | slider | — | — | Drag the slider around the grid to adjust the balance of frequency ranges. | Tone and Balance settings appear after you turn on Equalizer. |
| MD.10.c206 | Adjust the tone | Tone | popup | Broad; Focused | — | Choose Broad for a wider range of sounds, or Focused to hear specific sounds. | Shown after you turn on Equalizer. |
| MD.10.c207 | Adjust the sound balance | Balance | slider | — | — | Drag the slider left or right to adjust the sound balance. | Shown after you turn on Equalizer. |
| MD.10.c208 | Set a timer | Choose | button | — | — | Opens the Timer settings. | — |
| MD.10.c209 | Set a timer | Timer | toggle | — | — | Turns on the timer. | — |
| MD.10.c210 | Set a timer | Set Timer | popup | Options are not listed on the page. | — | Choose a timer option. | The selection determines whether you can set an amount of time or a specific time in the text box below. |
| MD.10.c211 | Set a timer | Time | field | — | — | Enter an amount of time or a specific time. | What you can enter depends on the Set Timer selection. |
| MD.10.c212 | Set a timer | Always use these settings | toggle | — | — | Use the selected settings as part of a daily routine, such as during downtime or focus time. | — |
| MD.10.c213 | Turn off background sounds when your Mac isn't in use | Turn off background sounds when your Mac isn't in use | toggle | — | — | Automatically turn off background sounds when your Mac goes to sleep, the screen saver starts, or the screen locks. | — |
| MD.10.c214 | Adjust the sound settings on your Mac | Change Sound settings | link | — | — | Opens information about changing other sound settings that may affect background sounds. | — |
| MD.10.c215 | Tip | Hearing | other | Menu bar; Control Center | — | Add the Hearing option to the menu bar or Control Center to play background sounds from the desktop. | — |

Notes: The page does not state a macOS version. The page does not list the available background sound names or timer options. The page does not state defaults for the controls.

### Accessibility > Switch Control: Change Switch Control settings for accessibility on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mh43181/27/mac/27) · topic `mh43181` · path: Apple menu > System Settings > Accessibility > Switch Control · "Open … for me" link: yes

Use Switch Control settings to turn on Switch Control, configure panels and typing, manage switches and auto scanning, and adjust navigation. Auto scanning is on by default.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.10.c216 | Switch Control | Switch Control | toggle | — | — | Turn on Switch Control to display the Switch Control panel. | Switch Control can also be turned off in Switch Control settings or by closing the Switch Control panel. |
| MD.10.c217 | Switch Control | Appearance | popup | Light; Dark | — | Choose an appearance for Switch Control panels. This setting is independent of whether you use a light or dark appearance on your Mac. | — |
| MD.10.c218 | Switch Control | Fade panel after inactivity | toggle | — | — | Fade the Switch Control panel after a specified period of inactivity. | Press any switch to show the panel again. |
| MD.10.c219 | Switch Control | Duration of inactivity | other | Up to 60 seconds | — | Set the period of inactivity after which to hide the Switch Control panel. | — |
| MD.10.c220 | Switch Control | Allow platform switching to control your computer | toggle | — | — | Use another Apple device—such as an iPhone, iPad, or Apple TV 4K—to control your Mac without pairing the switches with it. | The other device must already be set up to work with your switches. Your Mac and other devices must be on the same Wi-Fi network, and you must be signed in to the same Apple Account on every device. |
| MD.10.c221 | Switch Control | Panel Editor | button | — | — | Create custom panels to add to the list in the Switch Control Custom panel, or to share with others. | — |
| MD.10.c222 | Typing | Show current text in keyboard panel | toggle | — | — | Display a bar across the top of the Keyboard panel to show text as you’re typing. | — |
| MD.10.c223 | Typing | Insert and remove spaces automatically | toggle | — | — | Let Switch Control manage spaces when you’re writing text, such as correcting spacing after a period or between sentences based on context. | — |
| MD.10.c224 | Typing | Capitalize sentences automatically | toggle | — | — | Let Switch Control manage capitalization when you’re writing text, such as capitalizing the first letter in a sentence based on context. | — |
| MD.10.c225 | Typing | Keyboard Settings | link | — | — | Open Keyboard settings to turn on additional typing features, such as automatic correction and text substitutions. | — |
| MD.10.c226 | Switches | Required | list | — | — | Shows required switches. Switch Control requires the Select Item switch. When auto scanning is on, Switch Control automatically moves to the next item as it steps through a panel, group, or the user interface. | Before you can turn off auto scanning, you must create a Move To Next Item switch; Switch Control moves it into the Required list when you turn off auto scanning. |
| MD.10.c227 | Switches | Optional | list | — | — | Shows optional switches to use, regardless of whether auto scanning is on or off. | — |
| MD.10.c228 | Switches | Add , Remove | button | Add or remove an optional switch | — | Add or remove an optional switch. | — |
| MD.10.c229 | Switches | Edit pop-up menu | popup | — | — | Change the name or action for a switch, and choose what happens when you press and hold the switch. Click Reassign to use a different switch for the action. | — |
| MD.10.c230 | Switches | Auto scanning | toggle | — | On | Have Switch Control automatically step through, or scan, items in a panel, group, or the user interface. | To turn off auto scanning, you must add a switch for moving to the next item. |
| MD.10.c231 | Switches — Switch Timing | Hold before perform duration | other | — | — | Set the number of seconds that a switch must be pressed to be recognized. | — |
| MD.10.c232 | Switches — Switch Timing | Ignore switch repeats | other | — | — | Set the number of seconds during which repeated pressing of a switch is ignored. This is useful if you’re prone to accidentally pressing a switch multiple times. | — |
| MD.10.c233 | Switches — Switch Timing | Hold before repeat duration | other | — | — | Set the number of seconds that a switch must be pressed to be recognized a second time. | — |
| MD.10.c234 | Navigation | While navigating | popup | Do nothing; Speak selection; Play sounds; Speak & Play sounds | — | Choose what happens when an item is highlighted during a scan. | — |
| MD.10.c235 | Navigation | Restart action position | popup | — | — | Choose the position from which to start when you rescan an item, such as a panel, group, or window. | — |
| MD.10.c236 | Navigation | Resume auto scanning after selection | toggle | — | Off | Turn on this option to have auto scanning automatically continue after you select an item. By default, auto scanning is paused when you select an item, and you must press the Select Item switch to resume it. | — |
| MD.10.c237 | Navigation | Reverse cursor direction after hitting edge | toggle | — | — | Have the pointer bounce back when it reaches a boundary while scanning the user interface. | — |
| MD.10.c238 | Navigation | Switch Control cursor size | other | — | — | Choose the thickness of the border of the Switch Control cursor when scanning the user interface. | — |
| MD.10.c239 | Navigation | Pointer precision | popup | Low; High | — | Choose the precision with which you control the pointer when selecting an item on the screen. Low: when you press the Select Item switch, the blue line (for Move, Move & click, and Glide) or the pointer (for Rotate) stops. High: when you press the Select Item switch, the blue line or pointer substantially slows down so you can select more accurately where to stop it; when you press the switch again, it stops. | — |
| MD.10.c240 | Navigation | Loops | other | Up to 10 | — | Set the number of times a panel, group, or window is scanned. | — |
| MD.10.c241 | Navigation — Navigation Timing | Gliding & rotating cursor speed | other | 0 (slowest) to 100 (fastest) | — | Set the speed at which the blocks, sector, and blue lines move across the screen. | — |
| MD.10.c242 | Navigation — Navigation Timing | Auto scanning interval in panels | other | Up to 25 seconds | — | Set the number of seconds it takes to step from one item to another in a panel. | — |
| MD.10.c243 | Navigation — Navigation Timing | Auto scanning interval in interface | other | Up to 25 seconds | — | Set the number of seconds it takes to step from one item to another in the user interface. | — |
| MD.10.c244 | Navigation — Navigation Timing | Pause on first item | other | Up to 10 seconds | — | Set the number of seconds that auto scanning stays on the first item in a panel, group, or the user interface. | — |

Notes: The page refers to the Apple Support article “Use Switch Control on your device to control another Apple device.” The page refers to “Use the Panel Editor for Switch Control,” “Change the voice your Mac uses to speak text,” and Keyboard settings.

### Accessibility > Keyboard: Change Keyboard settings for accessibility on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlae61a6de/27/mac/27) · topic `mchlae61a6de` · path: Apple menu > System Settings > Accessibility > Keyboard · "Open … for me" link: yes

Use Keyboard settings for accessibility to configure Full Keyboard Access, Sticky Keys, Slow Keys, and the Accessibility Keyboard, including its dwell options.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.10.c245 | Keyboard settings for accessibility | Full Keyboard Access | toggle | — | — | Turn on Full Keyboard Access to use the Tab key and other keys to navigate the screen. | — |
| MD.10.c246 | Full Keyboard Access | Auto-hide | toggle | — | — | Turn on automatic hiding of the Full Keyboard Access cursor. | Applies when Full Keyboard Access is on. |
| MD.10.c247 | Full Keyboard Access | Auto-hide delay | other | Specified number of seconds | — | Automatically hide the Full Keyboard Access cursor after the specified number of seconds. | Applies when Full Keyboard Access is on. |
| MD.10.c248 | Full Keyboard Access | Increase size | toggle | — | — | Make the Full Keyboard Access cursor wider so it’s easier to see. | Applies when Full Keyboard Access is on. |
| MD.10.c249 | Full Keyboard Access | High contrast | toggle | — | — | Increase the contrast of the Full Keyboard Access cursor so it’s easier to see. | Applies when Full Keyboard Access is on. |
| MD.10.c250 | Full Keyboard Access | Color | other | Choose a color | — | Choose a color for the Full Keyboard Access cursor. | Applies when Full Keyboard Access is on. |
| MD.10.c251 | Full Keyboard Access | Full Keyboard Access commands | link | — | — | View a complete list of Full Keyboard Access commands. | — |
| MD.10.c252 | Full Keyboard Access | Customize keyboard shortcuts for default commands | other | — | — | Customize keyboard shortcuts for default commands. | — |
| MD.10.c253 | Full Keyboard Access | Set shortcuts for advanced commands | other | — | — | Set shortcuts for advanced commands. | — |
| MD.10.c254 | Full Keyboard Access | Restore default shortcuts | button | — | — | Restore the default Full Keyboard Access shortcuts. | — |
| MD.10.c255 | Keyboard settings for accessibility | Keyboard Settings | link | — | — | Set additional keyboard shortcuts, options for keyboard and function keys, and more. | — |
| MD.10.c256 | Keyboard settings for accessibility | Sticky Keys | toggle | — | — | Press a set of modifier keys one at a time in sequence, instead of all at once. | — |
| MD.10.c257 | Sticky Keys | Press the Shift key five times to toggle Sticky Keys | checkbox | — | — | Use your keyboard to turn Sticky Keys on or off. | — |
| MD.10.c258 | Sticky Keys | Beep when a modifier key is set | checkbox | — | — | Have your Mac make a sound when you press a modifier key. | — |
| MD.10.c259 | Sticky Keys | Display pressed keys on screen | checkbox | — | — | Display symbols to indicate which modifier keys you press. | — |
| MD.10.c260 | Sticky Keys | Screen area for display | other | Choose the area of the screen | — | Choose where to display the symbol for the pressed modifier key. | Available if “Display pressed keys on screen” is turned on. |
| MD.10.c261 | Keyboard settings for accessibility | Slow Keys | toggle | — | — | Adjust the delay between when you press a key and when it’s activated. | — |
| MD.10.c262 | Slow Keys | Use click key sounds | checkbox | — | — | Have keys make a sound when pressed. One sound plays when you press a key, and a different sound plays when the key press is accepted. | — |
| MD.10.c263 | Slow Keys | Acceptance delay | other | Set the delay | — | Set the delay between when you press a key and when it’s accepted. | — |
| MD.10.c264 | Keyboard settings for accessibility | Accessibility Keyboard | toggle | — | — | Display and use the Accessibility Keyboard, an onscreen keyboard that lets you type and interact without using a physical keyboard. | — |
| MD.10.c265 | Keyboard settings for accessibility | Panel Editor | button | — | — | Use the Panel Editor to create custom panels that are available from the Accessibility Keyboard. | — |
| MD.10.c266 | General options for the Accessibility Keyboard | Appearance | other | Light or dark | — | Use a light or dark appearance for the keyboard. This setting is independent of whether you use a light or dark appearance on your Mac. | — |
| MD.10.c267 | General options for the Accessibility Keyboard | Fade panel after inactivity | other | A specified period of inactivity; a specified fade percentage | — | After the specified period of inactivity, fade the keyboard by the specified percentage. At a higher percentage it’s less visible; at 100% it’s hidden. Move the pointer over the faded keyboard to make it fully visible again, or move the pointer if it’s hidden. | — |
| MD.10.c268 | General options for the Accessibility Keyboard | Play sounds for keys and dwell actions | toggle | — | — | Hear a sound when you select a key on the keyboard or when a dwell action is performed. Selecting the Caps key makes a sound even when this option is not selected. | — |
| MD.10.c269 | General options for the Accessibility Keyboard | Keys should be entered on | other | Mouse up or mouse down | — | Choose whether input is entered or action is taken on mouse up or mouse down. | — |
| MD.10.c270 | General options for the Accessibility Keyboard | Keys can be selected using right-click | toggle | — | — | Select keys on the keyboard using either a left click or right click of the mouse or trackpad. | — |
| MD.10.c271 | General options for the Accessibility Keyboard | Insert and remove spaces automatically | toggle | — | — | Let the Accessibility Keyboard manage spaces when you’re writing text, such as correcting spacing after a period or between sentences based on context. | — |
| MD.10.c272 | General options for the Accessibility Keyboard | Capitalize sentences automatically | toggle | — | — | Let the Accessibility Keyboard manage capitalization when you’re writing text, such as capitalizing letters after a period based on context. | — |
| MD.10.c273 | Dwell Corners options for the Accessibility Keyboard | Dwell Corners | other | Choose an action, such as Left Click | — | Choose the action to take when you keep the pointer in a corner of the screen. | — |
| MD.10.c274 | Dwell Corners options for the Accessibility Keyboard | Panel follows hide/show home panel | toggle | — | — | When you use a dwell corner to show or hide the Accessibility Keyboard or a custom panel, have the keyboard or panel move to the dwell corner. Turn this option off to keep it from moving there. | — |
| MD.10.c275 | Dwell options for the Accessibility Keyboard | Allow dwell actions toolbar in panels | toggle | — | — | Make the toolbar available in the Accessibility Keyboard and custom panels. | — |
| MD.10.c276 | Dwell options for the Accessibility Keyboard | Show dwell actions in menu bar | toggle | — | — | Show a status menu for dwell actions in the menu bar. | — |
| MD.10.c277 | Dwell options for the Accessibility Keyboard | Always dwell in menu extras | toggle | — | — | Always use Dwell to select items on the right side of the menu bar, even when Dwell is paused. Left-click is the default action when dwelling in menu bar extras. | — |
| MD.10.c278 | Dwell options for the Accessibility Keyboard | Always dwell in panels | toggle | — | — | Always use Dwell to select items in the Accessibility Keyboard and custom panels, even when Dwell is paused. Left-click is the default action when dwelling in panels. | — |
| MD.10.c279 | Dwell options for the Accessibility Keyboard | Dwell color | other | Choose a color, such as green | — | Choose the color of the dwell pointer. | — |
| MD.10.c280 | Dwell options for the Accessibility Keyboard | Enable zoom | toggle | — | — | Show the zoom window after the pointer is idle for the specified amount of time. The window magnifies the area you dwell on and the pointer. The zoom window size is based on the size set for Picture-in-Picture zoom. | The page advises avoiding changes to other options in Zoom settings for accessibility because they could interfere with the Dwell zoom window. |
| MD.10.c281 | Dwell options for the Accessibility Keyboard | Hide dwell time indicators | toggle | — | — | Hide the dwell pointer. | — |
| MD.10.c282 | Dwell options for the Accessibility Keyboard | Default dwell action | other | Right click; show the Scroll Menu; drag and drop an item; show the Options menu | — | Choose the default action to perform when you dwell on an item. The Options menu lets you switch between actions for one-time-only use. You can also choose an action from the dwell status menu if it’s showing in the menu bar. | — |
| MD.10.c283 | Dwell options for the Accessibility Keyboard | Auto revert to left click | toggle | — | — | After you select an action to perform, always make left click the next action performed. | — |
| MD.10.c284 | Dwell options for the Accessibility Keyboard | Default dwell time | other | — | — | Set the amount of time to dwell on an item on the screen before an action is performed. | — |
| MD.10.c285 | Dwell options for the Accessibility Keyboard | Panel dwell time | other | — | — | Set the amount of time to dwell on an item in a panel before a left-click action is performed. | — |
| MD.10.c286 | Dwell options for the Accessibility Keyboard | Dwell movement tolerance | other | — | — | Set the distance, in pixels, you can move while dwelling on an item. If you move beyond this limit, the countdown starts again. | — |
| MD.10.c287 | Dwell options for the Accessibility Keyboard | Post-action movement tolerance | other | — | — | Set the distance, in pixels, you need to move after performing an action for a new countdown to start. | — |
| MD.10.c288 | Accessibility Shortcuts panel | Specify which options are available in the panel | link | — | — | Choose which options are available in the Accessibility Shortcuts panel. | — |

Shortcuts stated: Option-Command-F5 turns Full Keyboard Access, Sticky Keys, Slow Keys, or the Accessibility Keyboard on or off using the Accessibility Shortcuts panel.; If your Mac or Magic Keyboard has Touch ID, quickly press Touch ID three times to open the Accessibility Shortcuts panel.

Notes: The page does not state a macOS version. The page does not specify defaults for the listed settings. The “Open Keyboard settings for accessibility for me” and “Open Shortcut settings for accessibility for me” links are present.

### Accessibility > Pointer Control: Change Pointer Control settings for accessibility on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/unac899/27/mac/27) · topic `unac899` · path: Apple menu > System Settings > Accessibility > Pointer Control · "Open … for me" link: no

Use Pointer Control settings to adjust mouse and trackpad behavior, enable Mouse Keys, configure alternate pointer actions, or move the pointer with head pointer. Available options depend on the Mac.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.10.c289 | Mouse & Trackpad | Double-click speed | slider | — | — | Adjust the mouse or trackpad’s reaction time when you double-click an item. | — |
| MD.10.c290 | Mouse & Trackpad | Spring-loading | toggle | — | — | When dragging and dropping, drag an item to a folder and hold to open the folder. | — |
| MD.10.c291 | Mouse & Trackpad | Spring-loading speed | slider | — | — | Adjust how long after you drag an item to a folder and hold before the folder opens. | With Spring-loading turned on. |
| MD.10.c292 | Mouse & Trackpad | Ignore built-in trackpad when mouse or wireless trackpad is present | checkbox | — | — | Disable the built-in trackpad on your Mac when you use a mouse or wireless trackpad. | If you use a non-Apple mouse, its driver software may prevent this option from working as expected. |
| MD.10.c293 | Mouse & Trackpad | Trackpad Options | button | — | — | Click to set options for your trackpad. | — |
| MD.10.c294 | Mouse & Trackpad > Trackpad Options | Use trackpad for scrolling | toggle | — | — | Turn scrolling on or off. | — |
| MD.10.c295 | Mouse & Trackpad > Trackpad Options | Use inertia when scrolling | toggle | — | — | Control what happens when you lift your finger from the trackpad. When turned on, scrolling comes to a gradual stop; otherwise, scrolling stops immediately. | — |
| MD.10.c296 | Mouse & Trackpad > Trackpad Options | Scroll speed | slider | — | — | Set the speed at which you scroll through documents and windows using the trackpad. | — |
| MD.10.c297 | Mouse & Trackpad > Trackpad Options | Use trackpad for dragging | toggle | — | — | Turn dragging on or off. | — |
| MD.10.c298 | Mouse & Trackpad > Trackpad Options | Dragging style | popup | Without Drag Lock; With Drag Lock; Three Finger Drag | — | Choose how dragging works. Without Drag Lock, double-tap an item and drag without lifting your finger after the second tap; dragging stops when you lift your finger. With Drag Lock, dragging continues when you lift your finger and stops when you tap the trackpad once. With Three Finger Drag, drag an item with three fingers; dragging stops when you lift your fingers. | Applies when trackpad dragging is used. |
| MD.10.c299 | Mouse & Trackpad | Mouse Options | button | — | — | Click to set options for your mouse. | — |
| MD.10.c300 | Mouse & Trackpad > Mouse Options | Use mouse for scrolling | toggle | — | — | Turn scrolling on or off, for a mouse such as a Magic Mouse. | — |
| MD.10.c301 | Mouse & Trackpad > Mouse Options | Use inertia when scrolling | toggle | — | — | Control what happens when you lift your finger from the mouse, for a mouse such as a Magic Mouse. When turned on, scrolling comes to a gradual stop; otherwise, scrolling stops immediately. | — |
| MD.10.c302 | Mouse & Trackpad > Mouse Options | Scroll speed | slider | — | — | Set the speed at which you scroll through documents and windows using the mouse. | — |
| MD.10.c303 | Alternate Control Methods | Mouse Keys | toggle | — | — | Move the pointer and press the mouse button using the keyboard or a numeric keypad. When Mouse Keys is on, you can’t use the keyboard or numeric keypad to enter text. Click to customize the options below. | While Mouse Keys is on, the keyboard or numeric keypad can’t be used to enter text. |
| MD.10.c304 | Alternate Control Methods > Mouse Keys | Press the Option key five times to toggle Mouse Keys | checkbox | — | — | Use your keyboard to turn Mouse Keys on or off. | — |
| MD.10.c305 | Alternate Control Methods > Mouse Keys | Ignore built-in trackpad when Mouse Keys is on | checkbox | — | — | Disable the trackpad on your Mac when you use Mouse Keys. | When Mouse Keys is on. |
| MD.10.c306 | Alternate Control Methods > Mouse Keys | Initial delay | slider | — | — | Set how quickly the pointer starts moving when you press a key. | — |
| MD.10.c307 | Alternate Control Methods > Mouse Keys | Maximum speed | slider | — | — | Set the maximum speed at which the pointer moves. | — |
| MD.10.c308 | Alternate Control Methods | Alternate pointer actions | toggle | — | — | Perform a left-click, right-click, double-click, or drag-and-drop action using keyboard keys, assistive switches, or facial expressions. Configure assigned keys, switches, or facial expressions in the list. Double-click an item to change its name or assigned action; select an item to remove it; click the add or remove control to add or remove an item. | Can use keyboard keys, assistive switches, or facial expressions. Facial-expression detection uses a camera built into or connected to your Mac.; A left-click, right-click, or double-click occurs as soon as you use the specified switch. Drag-and-drop requires using a switch twice: once to drag and again to drop. |
| MD.10.c309 | Alternate Control Methods > Alternate pointer actions | Play sounds | toggle | — | — | Hear confirmation that your switch is recognized. | — |
| MD.10.c310 | Alternate Control Methods > Alternate pointer actions | Show actions visually | toggle | — | — | See onscreen confirmation that your switch is recognized. | — |
| MD.10.c311 | Alternate Control Methods > Alternate pointer actions | Camera Options | button | — | — | Click to choose the camera used to detect facial expressions. | A camera built into or connected to your Mac is used for facial-expression detection. |
| MD.10.c312 | Alternate Control Methods > Alternate pointer actions > Camera Options | Camera | popup | — | — | Choose the camera to use to detect facial expressions. | — |
| MD.10.c313 | Alternate Control Methods > Alternate pointer actions | Pop-up menu for an expression | popup | Slight; Default; Exaggerated | — | Set the level of expressiveness the camera needs to detect before an action is taken. | The same camera and expressiveness level are used for alternate pointer actions and head pointer. |
| MD.10.c314 | Alternate Control Methods | Head pointer | toggle | — | — | Move the pointer using movement of your face or head as detected by the camera built into or connected to your Mac. | Requires a built-in or connected camera. |
| MD.10.c315 | Alternate Control Methods > Head pointer | Pointer moves | popup | When facing screen edges; Relative to head | — | Choose how the pointer follows your face or head movement. When facing screen edges, the pointer follows the direction you turn your face away from the center of the screen; turn your face back toward the center to stop it. Relative to head follows the direction you move your head, regardless of where your face is turned; stop moving your head to stop the pointer. | — |
| MD.10.c316 | Alternate Control Methods > Head pointer | Pointer speed | slider | — | — | Set the speed at which the pointer moves when you turn your face or move your head. A lower number is slower; a higher number is faster. | — |
| MD.10.c317 | Alternate Control Methods > Head pointer | Distance to edge | slider | — | — | Set how far your face must be turned away from the center of the screen before the pointer starts moving. A higher number can help the camera differentiate intentional from accidental movements. | Applies when Pointer moves is set to “When facing screen edges.” |
| MD.10.c318 | Alternate Control Methods > Head pointer | Use a switch or facial expression to pause or resume | other | — | — | Specify a facial expression to pause or resume head pointer. | Requires a switch or facial expression to be specified; facial-expression detection uses a camera.; Pausing head pointer can be useful if you’re going to be away from your Mac. |
| MD.10.c319 | Alternate Control Methods > Head pointer | Use a switch or facial expression to recalibrate | other | — | — | Specify a facial expression to recalibrate head pointer. | Requires a switch or facial expression to be specified; facial-expression detection uses a camera. |
| MD.10.c320 | Alternate Control Methods > Head pointer | Camera Options | button | — | — | Click to choose the camera used to detect movement of your face or head. | A built-in or connected camera is required for head pointer. |
| MD.10.c321 | Alternate Control Methods > Head pointer > Camera Options | Camera | popup | — | — | Choose the camera to use to detect movement of your face or head. | — |
| MD.10.c322 | Alternate Control Methods > Head pointer | Pop-up menu for an expression | popup | Slight; Default; Exaggerated | — | Set the level of expressiveness the camera needs to detect before an action is taken. | The same camera and expressiveness level are used for head pointer and alternate pointer actions. |

Shortcuts stated: Press the Option key five times to toggle Mouse Keys.; Accessibility Shortcuts can quickly turn Mouse Keys or head pointer on or off; no specific key combination is given.

Notes: The page does not specify a macOS version. The available Pointer Control options depend on the Mac. A non-Apple mouse driver may prevent “Ignore built-in trackpad when mouse or wireless trackpad is present” from working as expected. The page does not state defaults for the controls.

#### Independent macOS 27 reports for MD.10

| ID | Kind | Pane | Claim | Source |
|---|---|---|---|---|
| MD.10.x01 | identifier | Accessibility, Audio Descriptions | The page reports that no identifier had been found yet. | T4 [MacMost](https://macmost.com/mac-settings-links) |
| MD.10.x02 | identifier | Accessibility, Hearing Devices | The page reports that no identifier had been found yet. | T4 [MacMost](https://macmost.com/mac-settings-links) |
| MD.10.x03 | renamed | Accessibility > Subtitles and Captioning | The Captions category is replaced by Subtitles and Captioning. | T4 [geeksmodo](https://geeksmodo.com/macos-system-settings/) |
| MD.10.x04 | new | Accessibility > Subtitles and Captioning | Three controls were added to adjust when and where subtitles or captions appear. | T4 [geeksmodo](https://geeksmodo.com/macos-system-settings/) |
| MD.10.x05 | new | Accessibility | A new Show Borders option makes interface element boundaries easier to see; the article does not specify its location in System Settings. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MD.10.x06 | behavior | System Settings > Accessibility | In macOS 26 Tahoe, “reduce transparency” could force the hard-style title-bar divider across apps, while also switching off other transparency effects. | T4 [Ars Technica](https://arstechnica.com/gadgets/2026/09/macos-27-golden-gate-the-ars-technica-review/) |
| MD.10.x07 | behavior | Accessibility | Accessibility settings include vision, hearing, motor, and speech options, with examples such as voice commands, color filters, and screen zoom. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.10.x08 | new | Settings catalog > Accessibility settings (DDM) | New DDM settings configure organization-defined accessibility preferences, including display, text, motion, audio, and interaction options. | T4 [Microsoft Tech Community](https://techcommunity.microsoft.com/blog/IntuneCustomerSuccess/microsoft-intune-and-apple-os-27-new-settings-support-and-platform-changes/4559764) |

## 13.11 MD.11 Appearance

Status: **PARTIAL**. Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13).


### Appearance: Change Appearance settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp1225/27/mac/27) · topic `mchlp1225` · path: Apple menu > System Settings > Appearance · "Open … for me" link: yes

Use Appearance settings to change the overall look of macOS and choose how scroll bars are displayed and used.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.11.c001 | Appearance | Appearance | other | Light; Dark; Auto | — | Choose the appearance for buttons, menus, and windows. Light doesn’t change the appearance. Dark darkens it so content stands out while windows and controls recede. Auto switches between light and dark based on the Night Shift schedule, or sunrise and sunset if no schedule is set. Auto switches only after the Mac has been idle for at least a minute. | — |
| MD.11.c002 | Appearance | Liquid Glass | slider | Clear at the left; heaviest tinting at the right; intermediate settings in between | — | Adjust the appearance of app icons. | — |
| MD.11.c003 | Appearance | Color | other | Colors not specified | — | Choose the color for buttons, pop-up menus, and other UI controls. | — |
| MD.11.c004 | Appearance | Text highlight color | other | Colors not specified | — | Choose the color used to highlight selected text. | — |
| MD.11.c005 | Appearance | Icon & widget style | other | Default; Dark; Clear; Tinted. For Dark: Always or Auto. For Clear: Light, Dark, or Auto. | — | Choose the color, background, and behavior of icons and widgets. Default items are opaque, with a light or colored background depending on the app. Dark items are opaque and darkened. For Dark, choose Always to keep items in Dark style or Auto to switch between Dark and Light based on system appearance. Clear items are transparent. For Clear, choose Light, Dark, or Auto to switch between those appearances based on system appearance. Tinted items are opaque against a dark background; icons, text, and graphics use the color selected under Theme in the Appearance pane. | — |
| MD.11.c006 | Appearance | Folder color | other | Colors not specified | — | Choose the color for folder icons in the Finder. | Appears when Default, Dark, or Clear is selected for “Icon & widget style.” |
| MD.11.c007 | Appearance | Icon, widget, & folder color | other | Colors not specified | — | Choose the color for icons and widgets, and for folder icons in the Finder. | Appears when Tinted is selected for “Icon & widget style.” |
| MD.11.c008 | Appearance | Sidebar icon size | other | Small; Medium; Large | — | Choose the size of icons in the Finder sidebar and in app sidebars such as those in Mail, News, and Photos. Sidebar text size changes too. | — |
| MD.11.c009 | Appearance | Tint window background with wallpaper color | other | On or off | — | Allow colors from the wallpaper to subtly tint certain window areas, such as toolbars or status bars. | — |
| MD.11.c010 | Appearance | Show scroll bars | other | Automatically based on mouse or trackpad; When scrolling; Always | — | Control when scroll bars are shown in a window with content that doesn’t fit its current size. Automatically based on mouse or trackpad shows scroll bars depending on the device: if it allows gestures, scroll bars are hidden until scrolling starts; otherwise, they’re visible. When scrolling hides them until scrolling starts. Always keeps them visible. | — |
| MD.11.c011 | Appearance | Click in the scroll bar to | other | Jump to the next page; Jump to the spot that’s clicked | — | Choose whether clicking in the scroll bar moves content page by page or to the clicked location, such as the bottom of a document. | Holding Option while clicking the scroll bar temporarily switches between the scrolling methods. |

Shortcuts stated: Option key: Press and hold while clicking the scroll bar to momentarily switch between scrolling methods.

Notes: The page does not state a macOS version. The page links to instructions for changing Night Shift settings and increasing text and icon size.

#### Independent macOS 27 reports for MD.11

| ID | Kind | Pane | Claim | Source |
|---|---|---|---|---|
| MD.11.x01 | new | Appearance | System Settings adds a slider to adjust Liquid Glass transparency, from ultra-clear to fully tinted. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MD.11.x02 | new | Appearance | A new Liquid Glass slider lets users adjust interface transparency, from ultra-clear to more opaque and tinted, or choose an intermediate setting. | T4 [iGeeksBlog](https://www.igeeksblog.com/macos-27-features/) |
| MD.11.x03 | new | Appearance | macOS 27 adds a Liquid Glass appearance slider, expanding beyond the two styles available previously. | T4 [9to5Mac](https://9to5mac.com/2026/09/22/macos-27-gives-you-more-control-over-liquid-glass/) |
| MD.11.x04 | behavior | Appearance | The slider defaults to a medium setting. | T4 [9to5Mac](https://9to5mac.com/2026/09/22/macos-27-gives-you-more-control-over-liquid-glass/) |
| MD.11.x05 | new | Appearance | The system invites users to adjust the slider during initial setup. | T4 [9to5Mac](https://9to5mac.com/2026/09/22/macos-27-gives-you-more-control-over-liquid-glass/) |
| MD.11.x06 | behavior | Appearance | Users can adjust the slider later in System Settings. | T4 [9to5Mac](https://9to5mac.com/2026/09/22/macos-27-gives-you-more-control-over-liquid-glass/) |
| MD.11.x07 | behavior | Appearance | The slider provides a live preview of each setting. | T4 [9to5Mac](https://9to5mac.com/2026/09/22/macos-27-gives-you-more-control-over-liquid-glass/) |
| MD.11.x08 | behavior | Appearance | Three notches indicate the clearest, medium, and most opaque settings, and the slider can also be set between notches. | T4 [9to5Mac](https://9to5mac.com/2026/09/22/macos-27-gives-you-more-control-over-liquid-glass/) |
| MD.11.x09 | behavior | Appearance | The slider changes the appearance of the Dock, including how clearly the desktop wallpaper shows through. | T4 [9to5Mac](https://9to5mac.com/2026/09/22/macos-27-gives-you-more-control-over-liquid-glass/) |
| MD.11.x10 | behavior | Appearance | The slider changes how dim desktop widgets look. | T4 [9to5Mac](https://9to5mac.com/2026/09/22/macos-27-gives-you-more-control-over-liquid-glass/) |
| MD.11.x11 | behavior | Appearance | The slider changes the appearance of system Now Playing bars in apps such as Apple Music and Podcasts. | T4 [9to5Mac](https://9to5mac.com/2026/09/22/macos-27-gives-you-more-control-over-liquid-glass/) |
| MD.11.x12 | behavior | Appearance | The slider also affects sidebars, toolbars, and toolbar buttons in apps such as Photos and Safari. | T4 [9to5Mac](https://9to5mac.com/2026/09/22/macos-27-gives-you-more-control-over-liquid-glass/) |
| MD.11.x13 | behavior | Appearance | One slider controls Liquid Glass appearances across the system, including the Dock, widgets, and window chrome. | T4 [9to5Mac](https://9to5mac.com/2026/09/22/macos-27-gives-you-more-control-over-liquid-glass/) |
| MD.11.x14 | behavior | System Settings > Appearance | The Appearance pane has a “Liquid Glass” section with a slider for adjusting transparency. | T4 [Apple World Today](https://appleworld.today/2026/09/how-to-adjust-the-transparency-of-liquid-glass-in-macos-27/) |
| MD.11.x15 | behavior | System Settings > Appearance > Liquid Glass | Dragging the slider left gives a clearer, more transparent look; dragging it right increases frosting, tint, and opacity. | T4 [Apple World Today](https://appleworld.today/2026/09/how-to-adjust-the-transparency-of-liquid-glass-in-macos-27/) |
| MD.11.x16 | changed | System Settings › Appearance | The Liquid Glass setting is a slider from fully transparent to opaque for the whole interface. | T4 [publicspace.net](https://www.publicspace.net/blog/macos-27-menu-bar/) |
| MD.11.x17 | behavior | System Settings › Appearance | Moving the Liquid Glass slider toward opaque makes the glass effect frosted throughout the interface, including the menu bar when its background is shown. | T4 [publicspace.net](https://www.publicspace.net/blog/macos-27-menu-bar/) |
| MD.11.x18 | identifier | Appearance and Accessibility | The listed commands open Appearance and Accessibility. Accessibility includes deep links for Voice Over, Zoom, Display, Spoken Content, Descriptions, Hearing Devices, Audio, RTT, Captions, Live Captions, Voice Control, Keyboard, Pointer Control, Switch Control, Live Speech, Personal Voice, Siri, and Shortcut. The legacy Accessibility command and these deep links use the legacy universal-access pane; Audio, RTT, Captions, and Siri are identified as macOS 14 Sonoma options. | T2 [GitHub](https://github.com/bvanpeski/SystemPreferences/blob/main/macos_preferencepanes-Ventura.md) |
| MD.11.x19 | behavior | Appearance | Light, Dark, and Auto appearance modes are available in System Settings > Appearance. Auto switches between Light and Dark depending on the time of day. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.11.x20 | behavior | Appearance | Accent color is configured in System Settings > Appearance > Theme > Color and appears in buttons, selection highlights, and sidebar icons. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.11.x21 | new | Appearance | macOS 27 adds a Liquid Glass slider under System Settings > Appearance > Liquid Glass to adjust the glass appearance from clearer to more tinted. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |

## 13.12 MD.12 Apple Intelligence & Siri

Status: **PARTIAL**. Guide documents pane as "Siri"; Apple Intelligence master switch reported removed (GAP-B4-05, 06).


### Apple Intelligence & Siri: Change Intelligence & Siri settings in Screen Time on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl1885610b/27/mac/27) · topic `mchl1885610b` · path: Apple menu > System Settings > Screen Time > Content & Privacy > Intelligence & Siri · "Open … for me" link: no

Use Intelligence & Siri settings in Screen Time to restrict access to Apple Intelligence, Siri, and Math Results.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.12.c001 | Apple Intelligence | Apple Intelligence | toggle | — | — | Allow access to Apple Intelligence features, including Image Creation, Writing Tools, and Intelligence Extensions. | Availability depends on language and region. Apple Intelligence is available with Siri and device language set to Chinese (Simplified), Chinese (Traditional), Danish, Dutch, English, French, German, Italian, Japanese, Korean, Norwegian, Portuguese, Spanish, Swedish, Turkish, or Vietnamese. Some features and devices may not be available in all regions or languages. |
| MD.12.c002 | Siri | Siri & Dictation | toggle | — | — | Allow this type of content. Turn off an option to restrict this type of content. | Available options may vary depending on Siri version and country or region. |
| MD.12.c003 | Siri | Explicit Language in Siri and Dictionary | toggle | — | — | Allow this type of content. Turn off an option to restrict this type of content. | Available options may vary depending on Siri version and country or region. |
| MD.12.c004 | Siri | Web Search Content in Siri | toggle | — | — | Allow this type of content. Turn off an option to restrict this type of content. | Available options may vary depending on Siri version and country or region. |
| MD.12.c005 | Math Results | Math Results | toggle | — | — | Allow math results to be solved automatically in text and handwriting. | — |

Notes: The available options may vary depending on your Siri version and by country or region. Apple Intelligence on macOS 27 is not available in all languages or regions. Usage limits may apply. Siri AI is described as available in beta in iOS 27, iPadOS 27, macOS 27, watchOS 27, and visionOS 27, and requires an Apple Intelligence–enabled device set to a supported language. It is available in English to start; the page says Siri AI will not initially be available in the EU on iOS, iPadOS, and watchOS. Certain Apple Intelligence features that rely on server-side models are subject to daily usage limits; the page says limits may vary by feature, request complexity, system demand, system policies, and other factors.

### Siri settings: Siri settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl3fd7fc15/27/mac/27) · topic `mchl3fd7fc15` · path: System Settings > Siri · "Open … for me" link: no

Use Siri settings to turn on Siri and customize how it is activated, its language and voice, its responses, and related app and privacy options. The options shown depend on which version of Siri you’re using.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.12.c006 | Siri settings | Turn on Siri | toggle | — | — | Turn on Siri to do everyday tasks, get quick answers, and more. | Only appears if Siri is turned off.; If Siri is turned off and later turned on again, you may need to set up Siri again. |
| MD.12.c007 | Siri settings | Requests | popup | Activate Siri by saying “Hey Siri” or “Siri,” or turn off listening and click the Siri icon or use a keyboard shortcut. | — | Choose how you want to activate Siri. | “Hey Siri” or “Siri” availability depends on the device, language, and region. |
| MD.12.c008 | Siri settings | Language | popup | Languages and dialects (not specified) | — | Change the language and dialect to use with Siri. | Siri isn’t available in all languages or countries or regions. |
| MD.12.c009 | Siri settings | Voice | popup | Variations; Voices | — | Select a preferred voice for Siri. | Some languages may only have one option. |
| MD.12.c010 | Siri settings | About Improve Siri, Dictation & Privacy | link | — | — | Learn how Siri may send certain data to Apple to process and help respond to requests, and how you can control what data is stored. | — |
| MD.12.c011 | Siri settings | Responses | radio | Prefer Silent Responses; Automatic; Prefer Spoken Responses | — | Choose how Siri responds. | — |
| MD.12.c012 | Siri settings | Always Show Request | checkbox | — | — | Show a transcription of your speech onscreen. | — |
| MD.12.c013 | Siri App | Change Siri app settings | link | — | — | Open Siri app settings. | Only available with Siri AI (Beta). |
| MD.12.c014 | ChatGPT | ChatGPT | other | Use ChatGPT to help write text, create an image, answer questions, and more. | — | Choose to use ChatGPT. | Only available with Apple Intelligence. Apple Intelligence on macOS 27 is not available in all languages or regions. Usage limits may apply. |
| MD.12.c015 | ChatGPT | Sign In | button | — | — | Sign in to your ChatGPT account, then follow the onscreen instructions. | ChatGPT settings are only available with Apple Intelligence. |
| MD.12.c016 | ChatGPT | Upgrade to ChatGPT Plus | button | — | — | Upgrade to ChatGPT Plus. | ChatGPT settings are only available with Apple Intelligence. |
| MD.12.c017 | ChatGPT | Confirm Requests | toggle | — | — | Turn off to send requests to ChatGPT without confirming. | ChatGPT settings are only available with Apple Intelligence. |
| MD.12.c018 | Siri settings | Automatic Visual Look Up | toggle | — | — | Select whether to automatically identify objects and places that appear in screenshots. | Only available with Apple Intelligence. |
| MD.12.c019 | Siri settings | App Access | list | Apps (not specified) | — | Choose the apps Siri can learn from and show Siri suggestions for. | — |
| MD.12.c020 | Siri settings | Turn Off Siri | button | — | — | Turn off Siri. | — |

Notes: The page says you may need to scroll down in the System Settings sidebar to find Siri. The options you see depend on which version of Siri you’re using. Apple Intelligence is available only with Siri and device language set to one of the languages listed on the page; some features, regions, and devices may not be supported. Siri AI is described as a beta in iOS 27, iPadOS 27, macOS 27, watchOS 27, and visionOS 27, requiring an Apple Intelligence–enabled device set to a supported language. It is available in English to start; the page notes regional limits and possible daily usage limits for some features. The page mentions using a keyboard shortcut to activate Siri but does not give a specific shortcut.

### Apple Intelligence & Siri: Turn off and restrict access to Apple Intelligence features on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlb2e44f94/27/mac/27) · topic `mchlb2e44f94` · path: System Settings app (no more specific path stated) · "Open … for me" link: no

The page describes where to change or restrict access to Apple Intelligence features on Mac. It does not provide specific settings-pane controls, defaults, or restart requirements.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.12.c021 | Apple Intelligence features | Siri AI (Beta) | Not stated | — | — | Can be turned off. | Siri AI is available in beta on macOS 27 and requires an Apple Intelligence–enabled device set to a supported language. It is available in English to start. Apple Intelligence availability varies by language and region; usage limits may apply. |
| MD.12.c022 | Apple Intelligence features | Switch to Siri Classic | Not stated | Turn off Siri, click “Turn On Siri,” then select “Use Siri Classic.” | — | Use Siri Classic instead of Siri AI. | The instructions refer to turning off Siri first. |
| MD.12.c023 | Apple Intelligence features | Messages summaries | Not stated | Change in the Messages app. | — | Can be turned off. | — |
| MD.12.c024 | Apple Intelligence features | Email summaries | Not stated | Change in the Mail app. | — | Can be turned off. | — |
| MD.12.c025 | Apple Intelligence features | Notification summaries | Not stated | Change in the System Settings app. | — | Can be turned off. | — |
| MD.12.c026 | Apple Intelligence features | Personalized Smart Replies in Mail | Not stated | Change in the Mail app. | — | Can be turned off. | — |
| MD.12.c027 | Apple Intelligence features | Voicemail suggestions in Phone | Not stated | Change in the Phone app. | — | Can be turned off. | — |
| MD.12.c028 | Apple Intelligence features | Call Context cards | Not stated | See “Customize apps Siri has access to.” | — | The page directs readers to customize apps Siri has access to. | — |
| MD.12.c029 | Apple Intelligence features | Writing Prompts in Journal | Not stated | Change in the Journal app. | — | Can be turned off. | — |
| MD.12.c030 | Screen Time | Restrict access to Apple Intelligence features in Screen Time | Not stated | For yourself: see “Change Siri Restrictions settings in Screen Time.” For a child: see “Block or allow access and changes to features on your kid’s device” in Apple Devices for Kids and Families. | — | Describes where to find instructions for restricting access to Apple Intelligence features. | For a child, the page refers to the child’s device. |
| MD.12.c031 | Suggestions | Change how you see suggestions from your apps | Not stated | See “Customize apps Siri has access to.” | — | The page directs readers to customize how suggestions from apps are shown. | — |

Notes: The page does not state control defaults, specific control types, restart requirements, or confirmation dialogs. It says Apple Intelligence is available when Siri and the device language are set to a supported language; some features may not be available in all regions or languages. Supported languages listed are Chinese (Simplified), Chinese (Traditional), Danish, Dutch, English, French, German, Italian, Japanese, Korean, Norwegian, Portuguese, Spanish, Swedish, Turkish, and Vietnamese. Daily usage limits may apply to some server-side features.

#### Independent macOS 27 reports for MD.12

| ID | Kind | Pane | Claim | Source |
|---|---|---|---|---|
| MD.12.x01 | new | Siri | A new Voice section in System Settings lets users adjust Siri's expressiveness and speaking pace with sliders. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MD.12.x02 | removed | Apple Intelligence | A toggle that previously let users turn off Apple Intelligence and delete its downloaded models is no longer available. | T4 [Michael Tsai](https://mjtsai.com/blog/2026/09/14/macos-27-golden-gate/) |
| MD.12.x03 | new | System Settings > Siri | On Macs with an M3 or newer processor and at least 12GB of RAM, Siri settings include pace and expressivity sliders for its voice. | T4 [Ars Technica](https://arstechnica.com/gadgets/2026/09/macos-27-golden-gate-the-ars-technica-review/) |
| MD.12.x04 | behavior | System Settings > Siri | Pace controls how quickly Siri speaks, and expressivity controls how much inflection its voice uses. | T4 [Ars Technica](https://arstechnica.com/gadgets/2026/09/macos-27-golden-gate-the-ars-technica-review/) |
| MD.12.x05 | restriction | System Settings > Siri | Pace and expressivity options are currently available only for American and British accents; non-American accents have fewer voice options. | T4 [Ars Technica](https://arstechnica.com/gadgets/2026/09/macos-27-golden-gate-the-ars-technica-review/) |
| MD.12.x06 | behavior | Siri AI | Users can opt out of the Siri AI beta; the article says they are then offered the choice to turn Siri off or use “Siri Classic.” | T4 [Ars Technica](https://arstechnica.com/gadgets/2026/09/macos-27-golden-gate-the-ars-technica-review/) |
| MD.12.x07 | behavior | Siri | The pane appears with a new icon and the new version is available only when Language is set to English (United States); other languages, including English (Canada), show the old icon and Tahoe-era Siri features. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.12.x08 | removed | Siri | The new pane no longer has an Apple Intelligence switch or a Siri switch at the top. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.12.x09 | behavior | Siri | To disable Siri, scroll to the bottom, click the red Turn Off Siri button, read its description of what will stop working, and click Turn Off Siri again. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.12.x10 | behavior | Siri | When Siri is turned off, Visual Intelligence continues to work with more limited capabilities; Writing Tools, Image Playground, and Shortcuts continue to work. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.12.x11 | new | Siri | The Siri App setting Opens To selects Recent Conversations or A New Conversation. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.12.x12 | changed | Siri | Keep Conversations offers 30 Days, 1 Year, or Forever; the pane no longer has a Delete Siri & Dictation History button, and history items can instead be deleted in the Siri app. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.12.x13 | new | Siri | Preview offers None or 1–5 lines. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.12.x14 | renamed | Siri | Requests replaces the previous Listen for pop-up menu, whose choices included Siri, Hey Siri, or Off. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.12.x15 | changed | Siri | Responses offers Spoken Response or Silent Response; macOS Tahoe also offered Automatic, Prefer Silent Responses, and Prefer Spoken Responses. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.12.x16 | renamed | Siri | App Access replaces Siri Suggestions & Privacy for specifying each app’s Siri settings. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.12.x17 | new | Siri | The pane includes additional ChatGPT options for users who connect Siri to ChatGPT. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.12.x18 | moved | Siri | The Automatic Visual Look Up switch is in the Siri pane. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.12.x19 | removed | Siri | macOS 27 no longer lets users set the keyboard shortcut used to invoke Siri; macOS Tahoe did. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.12.x20 | behavior | Siri | Siri can be invoked with ⌘-Space, the Spotlight shortcut, or a second dedicated shortcut. Holding the dedicated shortcut speaks to Siri even when Requests is turned off. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.12.x21 | removed | System Settings > Apple Intelligence & Siri | The earlier single Apple Intelligence control at the top of this pane is no longer documented in macOS 27; users must change feature controls individually. | T5 [WindowsForum](https://windowsforum.com/news/macos-27-removes-apple-intelligence-master-toggle-wont-free-storage.445490/) |
| MD.12.x22 | behavior | System Settings > Siri | To turn off Siri AI (Beta), select “Turn Off Siri” and confirm by selecting it again. | T5 [WindowsForum](https://windowsforum.com/news/macos-27-removes-apple-intelligence-master-toggle-wont-free-storage.445490/) |
| MD.12.x23 | behavior | System Settings > Siri | After turning Siri off, users can select “Turn On Siri” and then choose “Use Siri Classic” to switch to the older assistant. | T5 [WindowsForum](https://windowsforum.com/news/macos-27-removes-apple-intelligence-master-toggle-wont-free-storage.445490/) |
| MD.12.x24 | behavior | System Settings > Siri | Call Context and suggestions from apps are controlled through Siri’s app-access settings rather than through a single global control. | T5 [WindowsForum](https://windowsforum.com/news/macos-27-removes-apple-intelligence-master-toggle-wont-free-storage.445490/) |
| MD.12.x25 | behavior | System Settings > Siri > App Access | A user-written walkthrough says app access can be limited per app by turning off “Learn from this application” and, when shown, “Show Siri Suggestions in application.” The article says the walkthrough is not Apple’s own wording. | T5 [WindowsForum](https://windowsforum.com/news/macos-27-removes-apple-intelligence-master-toggle-wont-free-storage.445490/) |
| MD.12.x26 | behavior | System Settings > Screen Time > Content & Privacy > Intelligence & Siri | The “Apple Intelligence” option controls access to features including Image Creation, Writing Tools, and Intelligence Extensions. | T5 [WindowsForum](https://windowsforum.com/news/macos-27-removes-apple-intelligence-master-toggle-wont-free-storage.445490/) |
| MD.12.x27 | behavior | System Settings > Screen Time > Content & Privacy > Intelligence & Siri | Separate Siri options control Siri & Dictation, explicit language, and web search content in Siri. | T5 [WindowsForum](https://windowsforum.com/news/macos-27-removes-apple-intelligence-master-toggle-wont-free-storage.445490/) |
| MD.12.x28 | changed | System Settings > Screen Time > Intelligence & Siri | The article notes that Screen Time labels on macOS 27 may differ slightly from Tahoe-era labels and advises checking the pane on the Mac. | T5 [WindowsForum](https://windowsforum.com/news/macos-27-removes-apple-intelligence-master-toggle-wont-free-storage.445490/) |
| MD.12.x29 | behavior | Apple Intelligence & Siri | On macOS 26 or earlier, Apple Intelligence can be disabled in System Settings > Apple Intelligence & Siri > Apple Intelligence by toggling it off. | T4 [PCMag UK](https://uk.pcmag.com/macos/167598/is-apple-intelligence-stealing-your-macs-storage-heres-how-to-turn-it-off) |
| MD.12.x30 | behavior | Siri | On macOS 27, System Settings > Siri offers the choices Don't Allow Siri or Siri Classic, depending on preference. | T4 [PCMag UK](https://uk.pcmag.com/macos/167598/is-apple-intelligence-stealing-your-macs-storage-heres-how-to-turn-it-off) |
| MD.12.x31 | behavior | Siri | On macOS 27, individual items in the Capabilities list can be turned off. | T4 [PCMag UK](https://uk.pcmag.com/macos/167598/is-apple-intelligence-stealing-your-macs-storage-heres-how-to-turn-it-off) |
| MD.12.x32 | behavior | Apple Intelligence & Siri | On macOS 26 or earlier, turning off Apple Intelligence should free disk space; a reboot may be needed before the available storage reflects the change. | T4 [PCMag UK](https://uk.pcmag.com/macos/167598/is-apple-intelligence-stealing-your-macs-storage-heres-how-to-turn-it-off) |
| MD.12.x33 | behavior | Siri | On macOS 27, disabling Apple Intelligence features and switching to classic Siri will not automatically clear storage. The article says System Data might decrease over time, but this is not guaranteed. | T4 [PCMag UK](https://uk.pcmag.com/macos/167598/is-apple-intelligence-stealing-your-macs-storage-heres-how-to-turn-it-off) |
| MD.12.x34 | identifier | Apple ID and Family | The listed commands open the Apple ID and Family sections; the Apple ID iCloud deep link is marked legacy, as is the alternate Family command. | T2 [GitHub](https://github.com/bvanpeski/SystemPreferences/blob/main/macos_preferencepanes-Ventura.md) |
| MD.12.x35 | identifier | Control Center and Siri & Spotlight | The listed commands open Control Center, Siri, and Spotlight; a legacy speech preference command is also provided. | T2 [GitHub](https://github.com/bvanpeski/SystemPreferences/blob/main/macos_preferencepanes-Ventura.md) |
| MD.12.x36 | behavior | System Settings → Siri → App Access | Learn from this App controls how Siri learns from app use to make suggestions; turning it off for an app stops Siri suggesting actions or content based on use of that app. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.12.x37 | behavior | System Settings → Siri | The Siri app saves conversations and syncs them across devices through iCloud. Retention choices are forever, one year, or 30 days. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.12.x38 | behavior | System Settings → Screen Time → Content & Privacy → Siri | Siri Classic switches back to the previous Siri. Content & Privacy must be turned on first if it is off. Screen Time can also restrict individual AI features. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.12.x39 | restriction | Siri AI on-screen awareness | There is no setting to turn off Siri AI's on-screen awareness. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.12.x40 | removed | System Settings → Apple Intelligence | The earlier single switch that turned Apple Intelligence off and deleted its downloaded models is gone in Golden Gate; the models remain on disk, though individual features can be restricted. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.12.x41 | new | Settings catalog > Siri settings (DDM) | New DDM settings control Siri availability and behavior, including Siri AI features, user-generated content, access while locked, profanity filtering, and reduction of sensitive content. | T4 [Microsoft Tech Community](https://techcommunity.microsoft.com/blog/IntuneCustomerSuccess/microsoft-intune-and-apple-os-27-new-settings-support-and-platform-changes/4559764) |
| MD.12.x42 | moved | MDM Restrictions payload > Siri Settings (DDM) | The listed Siri restriction settings are deprecated in the MDM Restrictions payload; the article gives Siri Settings as their DDM location. | T4 [Microsoft Tech Community](https://techcommunity.microsoft.com/blog/IntuneCustomerSuccess/microsoft-intune-and-apple-os-27-new-settings-support-and-platform-changes/4559764) |

Apple's restriction page lists per-feature controls (Siri AI (Beta), Switch to Siri Classic, Messages/Email/Notification summaries, Personalized Smart Replies, Call Context cards, and others), but it gives no single Apple Intelligence switch in the Siri pane ([Turn off and restrict access to Apple Intelligence features](https://support.apple.com/guide/mac-help/mchlb2e44f94/27/mac/27)). A single "Apple Intelligence" allow-toggle remains under Screen Time > Content & Privacy > Intelligence & Siri ([Intelligence & Siri in Screen Time](https://support.apple.com/guide/mac-help/mchl1885610b/27/mac/27)).

## 13.13 MD.13 Control Center

Status: **PARTIAL**. Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13).


### Control Center: Quickly change settings with Control Center on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlc9d0e1f2/27/mac/27) · topic `mchlc9d0e1f2` · path: Menu bar > Control Center > Edit Controls · "Open … for me" link: no

Open Control Center from the menu bar to adjust settings, toggle features, and access detailed options. You can customize which controls appear, rearrange or resize them, and pin controls to the menu bar.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.13.c001 | Control Center | Control Center | button | — | — | Click its icon in the menu bar to open a panel showing controls. Drag sliders, click icons, or click an arrow next to an item for more options. | — |
| MD.13.c002 | What’s in Control Center | Wi-Fi | other | Wireless networks | Included in Control Center by default. | Connect to wireless networks, switch between them, and see your current connection status. | — |
| MD.13.c003 | What’s in Control Center | Bluetooth | other | — | Included in Control Center by default. | Turn Bluetooth on or off and see which devices are currently paired or connected. | — |
| MD.13.c004 | What’s in Control Center | AirDrop | other | — | Included in Control Center by default. | Share files wirelessly without any setup. | Nearby iPhone, iPad, and Mac devices. |
| MD.13.c005 | What’s in Control Center | Focus | other | Do Not Disturb; Focus modes | Included in Control Center by default. | Turn on Do Not Disturb or switch between Focus modes to manage how and when you’re interrupted. | — |
| MD.13.c006 | What’s in Control Center | Stage Manager | other | — | Included in Control Center by default. | Toggle Stage Manager to organize open windows into a focused workspace, with other apps tucked to the side. | — |
| MD.13.c007 | What’s in Control Center | Screen Mirroring | other | Apple TV; AirPlay-compatible screen | Included in Control Center by default. | Mirror or extend your Mac display. | An Apple TV or AirPlay-compatible screen. |
| MD.13.c008 | What’s in Control Center | Display | other | Night Shift; True Tone | Included in Control Center by default. | Adjust screen brightness and quickly turn Night Shift or True Tone on or off. | — |
| MD.13.c009 | What’s in Control Center | Sound | other | Speakers or devices used for audio output | Included in Control Center by default. | Adjust the volume and switch which speaker or device your Mac uses for audio output. | — |
| MD.13.c010 | What’s in Control Center | Now Playing | other | Pause; skip; scrub | Included in Control Center by default. | Control media playback from any app without switching windows. | — |
| MD.13.c011 | Control Center | volume | slider | — | — | Drag a slider to adjust volume. | — |
| MD.13.c012 | Control Center | brightness | slider | — | — | Drag a slider to adjust brightness. | — |
| MD.13.c013 | Customize Control Center | Edit Controls | button | — | — | Choose this at the bottom of the Control Center panel to customize controls. | — |
| MD.13.c014 | Customize Control Center | Battery | other | — | — | A control that can be added to Control Center. | — |
| MD.13.c015 | Customize Control Center | Accessibility Shortcuts | other | — | — | A control that can be added to Control Center. | — |
| MD.13.c016 | Customize Control Center | Fast User Switching | other | — | — | A control that can be added to Control Center. | — |
| MD.13.c017 | Customize Control Center | Done | button | — | — | Click when you’re finished customizing. | — |
| MD.13.c018 | Customize Control Center | Add to Menu Bar | other | — | — | Choose this after Control-clicking a control to pin it to the menu bar. | — |
| MD.13.c019 | Customize Control Center | remove a menu bar icon | other | — | — | Hold Command and drag the icon off the menu bar to remove it. | — |
| MD.13.c020 | Customize Control Center | privacy indicator dots | other | — | — | Click any dot to see which app is currently using your microphone, camera, or system audio. | — |

Shortcuts stated: Control-click a control to resize it or choose Add to Menu Bar.; Hold Command and drag a menu bar icon off the bar to remove it.

Notes: The page does not specify a macOS version. Controls can be dragged in or out of Control Center and rearranged. The page does not state any restart, logout, confirmation, authentication, or data-loss effects.

#### Independent macOS 27 reports for MD.13

| ID | Kind | Pane | Claim | Source |
|---|---|---|---|---|
| MD.13.x01 | behavior | System Settings › Control Center | There is a switch for each of Apple’s listed menu bar icons, including Wi-Fi, Bluetooth, Battery, Sound, and Focus. | T4 [publicspace.net](https://www.publicspace.net/blog/macos-27-menu-bar/) |

## 13.14 MD.14 Desktop & Dock

Status: **PARTIAL**. Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13).


### Desktop & Dock: Change Desktop & Dock settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp1119/27/mac/27) · topic `mchlp1119` · path: Apple menu > System Settings > Desktop & Dock · "Open … for me" link: yes

Use Desktop & Dock settings to change the Dock’s appearance and choose settings for the desktop, Stage Manager, widgets, windows, and Mission Control.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.14.c001 | Dock | Size | slider | — | — | Drag the slider to change the Dock size. | — |
| MD.14.c002 | Dock | Magnification | slider | — | — | Magnify icons when you move the pointer over them. Drag the slider to choose the magnification size. | — |
| MD.14.c003 | Dock | Dock position on screen | popup | Left; Bottom; Right | — | Locate the Dock along the left, bottom, or right edge of the screen. | — |
| MD.14.c004 | Dock | Minimized windows animation | popup | — | — | Choose the visual effect used when you minimize a window. | — |
| MD.14.c005 | Dock | Window title bar double-click action | popup | Fill; Zoom; Minimize; No Action | — | Choose what happens when you double-click the title bar of an app window. Fill expands a window to fill the space between the menu bar and the Dock (or, when the Dock is along the side, the space below the menu bar and beside the Dock). Zoom zooms a window. Minimize minimizes an app window to the Dock. No Action does nothing. | — |
| MD.14.c006 | Dock | Minimize windows into application icon | checkbox | — | — | Minimize an app window to the app icon in the Dock. If this option isn’t turned on, the window is minimized to an area in the Dock. | — |
| MD.14.c007 | Dock | Automatically hide and show the Dock | checkbox | — | — | Hide the Dock when you’re not using it. To show the hidden Dock, move the pointer to the edge of the screen where the Dock is located. | — |
| MD.14.c008 | Dock | Animate opening applications | checkbox | — | — | Make icons bounce when you open apps. | — |
| MD.14.c009 | Dock | Show indicators for open applications | checkbox | — | — | Show a small dot below an app’s icon in the Dock when the app is open. | — |
| MD.14.c010 | Dock | Show suggested and recent apps in Dock | checkbox | — | — | Show suggested and recently opened apps (if they aren’t already included in the Dock) toward one end of the Dock. | — |
| MD.14.c011 | Desktop & Stage Manager | Show Items | checkbox | On Desktop; In Stage Manager | — | Choose whether to show desktop items. On Desktop shows items on the desktop. In Stage Manager shows items on the desktop when Stage Manager is turned on. If this option is turned off, items on the desktop are hidden; click the desktop to show them when you want to access them. | In Stage Manager applies when Stage Manager is turned on. |
| MD.14.c012 | Desktop & Stage Manager | Show Desktop | popup | Always on Wallpaper Click; Only in Stage Manager on Click | — | Choose when clicking the wallpaper moves windows out of the way. Always on Wallpaper Click moves all windows out of the way to show desktop items and widgets. Only in Stage Manager on Click does so when Stage Manager is turned on. | — |
| MD.14.c013 | Desktop & Stage Manager | Stage Manager | checkbox | — | — | Organize apps and windows so that your desktop is clutter-free and you can stay focused. | If this option is dimmed, turn on “Displays have separate Spaces” in the Mission Control section. |
| MD.14.c014 | Desktop & Stage Manager | Show recent apps in Stage Manager | checkbox | — | — | Show recent apps in Stage Manager. If this option is turned off, recently used apps are hidden; move the pointer to the left edge of the screen to briefly show them. | — |
| MD.14.c015 | Desktop & Stage Manager | Show windows from an application | popup | All at Once; One at a Time | — | Choose which windows to show when using an app. All at Once shows all available windows for an app when you switch to it. One at a Time shows only the most recently used window. To switch to another window when One at a Time is selected, click the app on the left again to open the next available window. | — |
| MD.14.c016 | Widgets | Show Widgets | checkbox | On Desktop; In Stage Manager | — | Choose whether to show widgets. On Desktop shows widgets on the desktop. In Stage Manager shows widgets on the desktop when Stage Manager is turned on. If this option is turned off, items on the desktop are hidden; click the desktop to show them when you want to access them. | In Stage Manager applies when Stage Manager is turned on. |
| MD.14.c017 | Widgets | Dim widgets on desktop | popup | Automatically; Always; Never | — | Choose a style for widgets. Automatically switches between monochrome and full-color. Always shows widgets in monochrome. Never shows widgets in full-color. | — |
| MD.14.c018 | Widgets | iPhone Widgets | checkbox | — | — | Show iPhone widgets in the widgets browser. | — |
| MD.14.c019 | Widgets | Default web browser | popup | Safari; another web browser installed on your Mac | Safari is used by default. | Choose another web browser you have installed on your Mac as the default. Safari is used by default when you click a link in an email or text message. | Another browser must be installed on your Mac to choose it. |
| MD.14.c020 | Windows | Prefer tabs when opening documents | popup | Never; Always; In Full Screen | — | Choose when you want documents to open in a tab instead of in a new window. | — |
| MD.14.c021 | Windows | Ask to keep changes when closing documents | checkbox | — | By default, unsaved changes are automatically saved when you close documents. | Turn on this option to be asked whether to save changes when closing documents. | — |
| MD.14.c022 | Windows | Close windows when quitting an application | checkbox | — | — | Close app windows when you quit an app. The windows don’t reopen automatically the next time you open the app. | — |
| MD.14.c023 | Windows | Drag windows to left or right edge of screen to tile | checkbox | — | — | Tile windows by dragging them to the left or right edge of the screen. | — |
| MD.14.c024 | Windows | Drag windows to menu bar to fill screen | checkbox | — | — | Fill the screen by dragging a window to the menu bar. | — |
| MD.14.c025 | Windows | Hold ⌥ key while dragging windows to tile | checkbox | — | — | Press the Option key to accelerate tiling when you drag windows to the edge of the screen. | — |
| MD.14.c026 | Windows | Tiled windows have margins | checkbox | — | — | Leave some space around each tiled window. | — |
| MD.14.c027 | Mission Control | Automatically rearrange Spaces based on most recent use | checkbox | — | — | Get quicker access to the desktops you used most recently, if you created additional desktops using Spaces. | Applies if you created additional desktops using Spaces. |
| MD.14.c028 | Mission Control | When switching to an application, switch to a Space with open windows for the application | checkbox | — | — | If you use Spaces, have your desktop scroll to a space with open windows for an app when you switch to that app. | Applies if you use Spaces. |
| MD.14.c029 | Mission Control | Group windows by application | checkbox | — | — | If you have several windows from multiple apps open simultaneously, have the windows organized by app in Mission Control. | Applies when several windows from multiple apps are open simultaneously. |
| MD.14.c030 | Mission Control | Displays have separate Spaces | checkbox | — | — | Set up separate spaces for each display. When turned on, the Dock is available on all displays; when turned off, the Dock appears only on the main display. This option must be turned on to use apps in Split View on other displays or to use Stage Manager. | Separate spaces apply if you use Spaces and have multiple displays. |
| MD.14.c031 | Mission Control | Drag windows to top of screen to enter Mission Control | checkbox | — | — | Enter Mission Control by dragging a window to the top of the screen. | — |
| MD.14.c032 | Mission Control | Shortcuts | other | — | — | View or change keyboard shortcuts for showing and hiding Mission Control, open windows of the current app, and the desktop. Depending on your mouse or trackpad, two columns of pop-up menus may be shown. Press and hold Control, Option, Shift, or Command, or a combination of these keys, while viewing the pop-up menus to display additional available keyboard shortcuts. | The displayed columns depend on your mouse or trackpad. |
| MD.14.c033 | Mission Control | Hot Corners | popup | Actions to perform when the pointer is moved to a screen corner; specific actions not listed | — | Choose actions to perform when you move the pointer to screen corners. To set up a hot corner, click the pop-up menu for a corner, then choose an action. To use a modifier key with a hot corner, press and hold Command, Shift, Option, or Control, or a combination of these keys, while viewing the pop-up menus. | — |

Shortcuts stated: Hold the ⌥ (Option) key while dragging windows to the edge of the screen to accelerate tiling.; Press and hold Control, Option, Shift, or Command, or a combination of these keys, while viewing Mission Control shortcut pop-up menus to display additional available shortcuts.; Press and hold Command, Shift, Option, or Control, or a combination of these keys, while viewing Hot Corners pop-up menus to use a modifier key with a hot corner.

Notes: The page does not state a macOS version. The navigation text is incomplete in the supplied page excerpt; the path is represented as Apple menu > System Settings > Desktop & Dock. No restart, logout, authentication prompt, confirmation dialog, or data-loss effect is stated.

### Widgets: Add and customize widgets on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl52be5da5/27/mac/27) · topic `mchl52be5da5` · path: System Settings > Desktop & Dock > Widgets · "Open … for me" link: yes

Add widgets to the desktop or Notification Center, use eligible iPhone widgets, customize or remove widgets, and change widget display settings.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.14.c034 | Introduction | date and time in the menu bar | button | — | — | Click to open Notification Center. | — |
| MD.14.c035 | Introduction | swipe left with two fingers from the right edge of the trackpad | other | — | — | Gesture to open Notification Center. | Requires a trackpad. |
| MD.14.c036 | Introduction | click anywhere on the desktop | other | — | — | Closes Notification Center. | — |
| MD.14.c037 | Introduction | click the wallpaper | other | — | — | Moves open windows away to reveal desktop widgets. | Open windows move away. |
| MD.14.c038 | Introduction | “Click wallpaper to reveal desktop” | popup | Only in Stage Manager | — | Changes when clicking the desktop moves open windows away. With “Only in Stage Manager,” clicking the desktop moves windows away only when Stage Manager is in use. | — |
| MD.14.c039 | Add widgets to the desktop | Edit Widgets | button | — | — | Opens the widget gallery from the desktop context menu. | Control-click the wallpaper first. |
| MD.14.c040 | Add widgets to the desktop | search for a widget | field | — | — | Searches for a widget in the widget gallery. | — |
| MD.14.c041 | Add widgets to the desktop | category | list | — | — | Click a category to view its available widgets. | — |
| MD.14.c042 | Add widgets to the desktop | widget | button | — | — | Click the widget, or its Add button, to automatically position it on the desktop. Alternatively, drag it to a position on the desktop. | The widget is added to the desktop. |
| MD.14.c043 | Add widgets to the desktop | Remove button | button | — | — | Removes a newly added widget if you decide you do not want it. | The widget is removed. |
| MD.14.c044 | Add widgets to the desktop | Done | button | — | — | Finishes adding widgets and closes the widget gallery. | — |
| MD.14.c045 | Add widgets to Notification Center | date and time in the menu bar | button | — | — | Opens Notification Center. | — |
| MD.14.c046 | Add widgets to Notification Center | Edit Widgets | button | — | — | Opens the widget gallery at the bottom of Notification Center. | — |
| MD.14.c047 | Add widgets to Notification Center | search for a widget | field | — | — | Searches for a widget in the widget gallery. | — |
| MD.14.c048 | Add widgets to Notification Center | category | list | — | — | Click a category to view its available widgets. | — |
| MD.14.c049 | Add widgets to Notification Center | widget | button | — | — | Click the widget or its Add button, or drag it to the upper-right corner of the desktop, to add it to Notification Center. Drag an added widget up or down to change its location. | The widget is added to Notification Center. |
| MD.14.c050 | Add widgets to Notification Center | Remove button | button | — | — | Removes a widget you do not want. | The widget is removed. |
| MD.14.c051 | Add widgets to Notification Center | Done | button | — | — | Finishes adding widgets and closes the widget gallery. | — |
| MD.14.c052 | Use iPhone widgets on your Mac | Desktop & Dock | link | — | — | Opens Desktop & Dock in System Settings. | — |
| MD.14.c053 | Use iPhone widgets on your Mac | “Use iPhone widgets.” | toggle | On; off | — | Makes iPhone widgets available in the widget gallery to add to the desktop or Notification Center. | iPhone must use iOS 17 or later; the same Apple Account must be used on iPhone and Mac; iPhone must be near the Mac or on the same Wi-Fi network.; Some widgets may prompt “Open [app name] on your iPhone to continue” if the app is not installed on the Mac; open the app on the iPhone to interact with it. Some third-party apps aren’t available for both iPhone and Mac. |
| MD.14.c054 | Customize widgets | Edit [widget name] | other | — | — | Opens options to change the information shown by the widget; options may also be changed by clicking highlighted information. Click Done in the widget when finished. | Available only when Edit [widget name] appears in the shortcut menu; some widgets cannot be customized this way. |
| MD.14.c055 | Customize widgets | Done | button | — | — | Finishes editing a widget. | — |
| MD.14.c056 | Customize widgets | different size | other | — | — | Choose a different size for the widget; specific sizes are not listed. | — |
| MD.14.c057 | Customize widgets | Remove Widget | other | — | — | Removes the widget from the shortcut menu. | The widget is removed. |
| MD.14.c058 | Remove widgets from the desktop | Edit Widgets | button | — | — | Opens the widget gallery after clicking the wallpaper. | — |
| MD.14.c059 | Remove widgets from the desktop | Remove button | button | — | — | Removes a widget from the desktop. | The widget is removed. |
| MD.14.c060 | Remove widgets from Notification Center | Remove Widget | other | — | — | Removes a widget from the Control-click shortcut menu. | The widget is removed. |
| MD.14.c061 | Remove widgets from Notification Center | Option key and remove button | button | — | — | Move the pointer over a widget, press and hold the Option key, then click the remove button. | The widget is removed. |
| MD.14.c062 | Change widget settings | Desktop & Dock | link | — | — | Opens Desktop & Dock in System Settings. | — |
| MD.14.c063 | Change widget settings | “On Desktop:” | checkbox | Selected; deselected | — | Show widgets on the desktop. | — |
| MD.14.c064 | Change widget settings | “In Stage Manager:” | checkbox | Selected; deselected | — | Show widgets when Stage Manager is turned on. If turned off, items on the desktop are hidden; click the desktop to show them when you want to access them. | — |
| MD.14.c065 | Change widget settings | “Dim widgets on desktop” | popup | “Automatically”; “Always”; “Never” | — | “Automatically” switches between monochrome and full-color; “Always” shows widgets in monochrome; “Never” shows widgets in full-color. | — |
| MD.14.c066 | Change widget settings | iPhone Widgets | toggle | On; off | — | Turns iPhone Widgets on or off. | Using iPhone widgets requires the iPhone, Apple Account, and network or proximity requirements stated above. |

Shortcuts stated: Swipe left with two fingers from the right edge of the trackpad; Control-click the wallpaper; Control-click a widget; Press and hold the Option key, then click the remove button

Notes: The page does not state which macOS version it applies to. The page does not state defaults for the settings.

#### Independent macOS 27 reports for MD.14

| ID | Kind | Pane | Claim | Source |
|---|---|---|---|---|
| MD.14.x01 | identifier | Notifications, Sound, Focus, and Screen Time | The listed commands open these settings sections. The alternate Notifications, Sound, and Screen Time commands are marked legacy. | T2 [GitHub](https://github.com/bvanpeski/SystemPreferences/blob/main/macos_preferencepanes-Ventura.md) |
| MD.14.x02 | behavior | Desktop & Dock | Dock settings include size, position, minimizing when not in use, and app-opening animation options. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.14.x03 | behavior | Desktop & Dock > Hot Corners | Hot Corners can be assigned actions such as Mission Control, Desktop view, or Screen Saver; moving the cursor to a corner activates the assigned function. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.14.x04 | behavior | Focus | Focus modes can be created from presets or with Custom, then configured with a name, icon, and color. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.14.x05 | behavior | Focus | Focus settings can specify which people and apps may send notifications and whether Time-Sensitive Notifications are allowed. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.14.x06 | behavior | Focus | Focus Filters can change app behavior, including showing or hiding Mail accounts, Calendar sets, Safari tab groups, or Messages accounts. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.14.x07 | behavior | Focus | Focus options include dimming the Lock Screen, hiding notification badges, or showing only certain widgets while Focus is active. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.14.x08 | behavior | Focus | Focus can be scheduled by time, location, or app. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.14.x09 | behavior | Focus | Share across devices can sync Focus settings with an iPhone, iPad, and Apple Watch; Focus status can tell contacts that notifications are silenced. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.14.x10 | behavior | Focus | On supported Macs, Reduce Interruptions with Apple Intelligence can intelligently filter notifications. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.14.x11 | behavior | Focus | The article says an app-based Focus trigger enables Focus when the selected app launches and disables it when the app closes or the user switches away. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.14.x12 | behavior | Desktop & Dock | Mission Control settings are located in System Settings > Desktop & Dock. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |

## 13.15 MD.15 Displays

Status: **PARTIAL**. Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13).


### Displays > Night Shift: Change Night Shift settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl9d976e85/27/mac/27) · topic `mchl9d976e85` · path: Apple menu > System Settings > Displays > Night Shift · "Open … for me" link: no

Use Night Shift in Displays settings to shift your display to warmer colors, which can be easier on your eyes at night or in low-light conditions.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.15.c001 | Night Shift | Schedule | popup | Not listed | — | Choose an option to turn on Night Shift automatically at certain times. | — |
| MD.15.c002 | Night Shift | Turn on until tomorrow | checkbox | — | — | Turn Night Shift on until the next day or until you turn it off. | — |
| MD.15.c003 | Night Shift | Color temperature | other | — | — | Change the color temperature Night Shift uses. | — |

### Displays: Displays settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mh40768/27/mac/27) · topic `mh40768` · path: Apple menu > System Settings > Displays · "Open … for me" link: no

Use Displays settings to adjust resolution and brightness and set display options. Not all options appear for all displays or Mac models; some appear only when a Mac is connected to a display.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.15.c004 | Displays settings | Arrange | other | — | — | Arrange the position of the displays, mirror their content, or relocate the position of the menu bar. | Appears only when an external display is connected to your Mac. |
| MD.15.c005 | Displays settings | pop-up menu | popup | — | — | Choose a display to mirror or extend your desktop. | — |
| MD.15.c006 | Displays settings | Use as | other | Mirror your desktop; extend your desktop; act as your main display | — | Choose whether your display mirrors or extends your desktop, or acts as your main display. | Appears only when an external display is connected to your Mac. |
| MD.15.c007 | Displays settings | Optimize for | other | External display; your desktop | — | Choose to optimize for the external display or your desktop. | Appears only when you are mirroring your display to an external display. |
| MD.15.c008 | Displays settings | Resolution | other | Default; another available resolution | Default (automatically uses the best resolution for the display) | Choose the amount of detail your display shows. Choose Default to automatically use the best resolution for the display, or choose another option to manually select a resolution. | — |
| MD.15.c009 | Displays settings | Show all resolutions | other | — | — | View all available resolutions, including low-resolution modes. | — |
| MD.15.c010 | Displays settings | Brightness | slider | — | — | Adjust how light or dark your screen appears. Move the slider to the right to make the screen brighter or to the left to make it darker. | — |
| MD.15.c011 | Displays settings | Automatically adjust brightness or Ambient light compensation (depending on your display) | other | — | — | Adjust the display brightness automatically based on current ambient lighting conditions. | Available only if your Mac has ambient light sensing. |
| MD.15.c012 | Displays settings | True Tone | other | — | — | Automatically adjust the colors on the display based on current ambient lighting conditions. | — |
| MD.15.c013 | Displays settings | Color profile | other | — | — | Change the color profile your display uses. Detailed information about installed color profiles is provided in ColorSync Utility. You can also create a custom color profile by calibrating your display. | — |
| MD.15.c014 | Displays settings | When connected to TV | other | Choose what to share; ask each time you connect | — | Choose what to share when connected to a TV or whether to ask each time you connect. | — |
| MD.15.c015 | Displays settings | Preset | other | Built-in reference modes | — | Choose a built-in reference mode that matches your media workflow. | — |
| MD.15.c016 | Displays settings | Customize Presets | other | — | — | Create custom reference modes. | — |
| MD.15.c017 | Displays settings | Calibrate Display | other | — | — | Use the Pro Display Calibrator for in-field recalibration for specific color workflows that may require custom calibration. | — |
| MD.15.c018 | Displays settings | Rotate or Rotation(depending on your display) | other | Chosen amount | — | Rotate the image shown on your display by the chosen amount. | Appears only if a compatible external display is connected to your Mac. |
| MD.15.c019 | Displays settings | Refresh rate | other | Available refresh rates; Adaptive Sync on a compatible display | — | Adjust how frequently your screen is redrawn. If the rate is too low, your screen may appear to flicker. If it is too high, the screen may be black because it doesn’t support that rate. On a compatible display, set the display to dynamically adjust its refresh rate using Adaptive Sync. | Appears only if an external display that supports changing the refresh rate is connected to your Mac. Adaptive Sync requires a compatible display.; An unsupported refresh rate may cause the screen to be black. |
| MD.15.c020 | Displays settings | Show Sidebar | other | Left; right; not at all | — | Choose whether the sidebar on your iPad appears on the left, the right, or not at all. | Appears only if you’re using your iPad as a second display for your Mac. |
| MD.15.c021 | Displays settings | Show Touch Bar | other | Top; bottom; not at all | — | Choose whether the Touch Bar on your iPad appears on the top, the bottom, or not at all. In an app that supports the Touch Bar, it appears in the specified location; its buttons vary by app and task. | Appears only if you’re using your iPad as a second display for your Mac. |
| MD.15.c022 | Displays settings | Enable double tap on Apple Pencil | checkbox | — | — | Select the checkbox to use a double-tap on the lower section of Apple Pencil to switch drawing tools in some apps. | Appears only if you’re using your iPad as a second display for your Mac; requires a compatible Apple Pencil. |
| MD.15.c023 | Displays settings | Underscan | slider | — | — | Use the slider if you are using a TV as a display and you can’t see the menu bar. | Appears only on older TVs. |
| MD.15.c024 | Displays settings | Dynamic resolution | other | — | — | Enable to allow the resolution of a virtual display to automatically match the size of its screen share viewer window. | Appears only when a Screen Sharing Virtual Display is connected. |
| MD.15.c025 | Displays settings | High dynamic range | other | High Dynamic Range (HDR) Video (P3-ST 2084) reference preset | — | Enable to allow remote ultra-high-definition video production workflows up to 1000 nits by supporting the High Dynamic Range (HDR) Video (P3-ST 2084) reference preset. | Appears only when a Screen Sharing Virtual Display is connected. |
| MD.15.c026 | Displays settings | Advanced | other | — | — | Set options for viewing resolutions and connecting to a nearby Mac or iPad. | — |
| MD.15.c027 | Advanced | Show resolutions as list | other | — | — | Enable to always see available resolutions as a list. | — |
| MD.15.c028 | Advanced | Link to Mac or iPad | other | — | — | Connect to a nearby Mac or iPad. | — |
| MD.15.c029 | Displays settings | Night Shift | other | — | — | Shift your display to the warmer end of the color spectrum. Warmer screen colors are easier on your eyes when you use your Mac at night or in low-light conditions. | — |
| MD.15.c030 | Displays settings | Detect Displays | button | — | — | Scan for all the displays connected to your Mac. | To make this button appear in place of the Night Shift button, press and hold the Option key. Use it if your Mac doesn’t recognize a display you just connected. |

Shortcuts stated: Press and hold the Option key to make Detect Displays appear in place of the Night Shift button.

Notes: The page does not state a macOS version. Not all options described appear for all displays or Mac models, and some appear only when the Mac is connected to a display. The page’s navigation instruction is truncated after “To open these settings, choose Apple menu”; the path is completed as Apple menu > System Settings > Displays based on the pane described.

#### Independent macOS 27 reports for MD.15

| ID | Kind | Pane | Claim | Source |
|---|---|---|---|---|
| MD.15.x01 | identifier | Displays | The System Settings link ending in `com.apple.Displays-Settings.extension` opens Displays. | T4 [Unpixel](https://unpixel.tech/blog/url-schemes-mac/) |
| MD.15.x02 | new | Accessibility > Display | A new switch adds borders around toolbar controls, Search fields, and other interface items to make their boundaries clearer. | T4 [geeksmodo](https://geeksmodo.com/macos-system-settings/) |
| MD.15.x03 | changed | Accessibility > Display | “Show borders” replaces the previous setting for toolbar button shapes. | T4 [geeksmodo](https://geeksmodo.com/macos-system-settings/) |
| MD.15.x04 | behavior | System Settings > Accessibility > Display | Turning on “Reduce Transparency” makes transparency almost completely off, producing a solid background. | T4 [Apple World Today](https://appleworld.today/2026/09/how-to-adjust-the-transparency-of-liquid-glass-in-macos-27/) |
| MD.15.x05 | behavior | System Settings › Accessibility › Display | Reduce transparency replaces blur with a flat fill across the interface: light grey in light mode and near-black in dark mode. | T4 [publicspace.net](https://www.publicspace.net/blog/macos-27-menu-bar/) |
| MD.15.x06 | behavior | System Settings › Displays | A scaled resolution that does not use the notch rows blacks out that strip, at the cost of those rows of screen. | T4 [publicspace.net](https://www.publicspace.net/blog/macos-27-menu-bar/) |
| MD.15.x07 | behavior | Displays | On macOS 26 Tahoe, an HDR10-capable third-party display has a High Dynamic Range toggle in Settings → Displays. | T4 [9to5Mac](https://9to5mac.com/2026/10/05/macos-27-just-fixed-the-most-annoying-problem-for-third-party-hdr-displays/) |
| MD.15.x08 | behavior | Displays | On macOS 26 Tahoe, selecting a scaled resolution such as 2560 × 1440 disables HDR and switches the display back to Standard Dynamic Range. | T4 [9to5Mac](https://9to5mac.com/2026/10/05/macos-27-just-fixed-the-most-annoying-problem-for-third-party-hdr-displays/) |
| MD.15.x09 | changed | Displays | macOS 27 Golden Gate allows HDR to remain enabled while a scaled resolution such as 2560 × 1440 is selected; the article reports this worked on an M4 MacBook Pro. | T4 [9to5Mac](https://9to5mac.com/2026/10/05/macos-27-just-fixed-the-most-annoying-problem-for-third-party-hdr-displays/) |
| MD.15.x10 | restriction | Displays | The author could not select both HDR and a scaled resolution on an M1 Max MacBook Pro running macOS 27, suggesting hardware requirements. | T4 [9to5Mac](https://9to5mac.com/2026/10/05/macos-27-just-fixed-the-most-annoying-problem-for-third-party-hdr-displays/) |
| MD.15.x11 | behavior | Displays > Night Shift | Night Shift can be scheduled to shift display colors toward the warmer end of the spectrum after dark. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |

## 13.16 MD.16 Spotlight

Status: **PARTIAL**. Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13).


### Spotlight: Spotlight settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl54d95e8a/27/mac/27) · topic `mchl54d95e8a` · path: System Settings > Spotlight · "Open … for me" link: no

Use Spotlight settings to change the categories that appear in Spotlight search results.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.16.c001 | Spotlight settings | Show Related Content | toggle | — | — | Allow content from Apple partners to be shown when searching or when looking up text or objects and photos. For more information, click About Search & Privacy. | — |
| MD.16.c002 | Spotlight settings | Reset Quick Keys | button | — | — | Restore Spotlight quick keys to their default settings. | — |
| MD.16.c003 | Spotlight Search History | Delete Search History | button | — | — | Delete all history associated with Spotlight search. | Click Delete Search History, then click Delete to confirm. |
| MD.16.c004 | Spotlight Search History | Delete | button | — | — | Confirm deletion of all history associated with Spotlight search. | Deletes Spotlight search history. |
| MD.16.c005 | Spotlight settings | Help Apple Improve Search | toggle | — | — | Allow Apple to store Safari, Siri, Spotlight, Lookup and #images search queries to help improve Search. The information collected is stored in a way that does not identify you and is used to improve search results. Searches include general knowledge queries and requests to do things like play music and get directions. For more information, click About Search Privacy or see the Apple Privacy website. | — |
| MD.16.c006 | Results from Apps | Results from Apps | list | Apps (not individually listed) | — | Turn on apps if you want them and their content to appear in Spotlight search results. | — |
| MD.16.c007 | Results from System | Results from System | list | Items (not individually listed) | — | Turn on an item if you want it and its content to appear in Spotlight search results. | — |
| MD.16.c008 | Spotlight settings | Results from Clipboard | toggle | — | — | Allow the contents of your Clipboard to appear in Spotlight search results. | — |
| MD.16.c009 | Spotlight settings | Clipboard history is available in Spotlight | popup | 30 minutes; 8 hours; 7 days | — | Choose how long Clipboard history is available in Spotlight. Clipboard history older than the selected timeframe won’t be visible in Spotlight. | — |
| MD.16.c010 | Spotlight settings | Clear Clipboard History | button | — | — | Remove your Clipboard history from Spotlight search results on your device. | Click Clear Clipboard History, then click Clear to confirm. |
| MD.16.c011 | Clear Clipboard History | Clear | button | — | — | Confirm removal of Clipboard history from Spotlight search results on your device. | Clears Clipboard history from Spotlight search results on your device. |
| MD.16.c012 | Spotlight settings | Search Privacy | button | — | — | Exclude specific files and folders from Spotlight searches. | — |
| MD.16.c013 | Search Privacy | About Search & Privacy | info | — | — | Learn how Spotlight searches may send certain data to Apple to process and help respond to your requests, and how you can control what data is stored. | Available when you click Search Privacy. |

## 13.17 MD.17 Wallpaper

Status: **PARTIAL**. Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13).


### Wallpaper: Wallpaper settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp1103/27/mac/27) · topic `mchlp1103` · path: Apple menu > System Settings > Wallpaper · "Open … for me" link: no

Use Wallpaper settings to change the picture or color shown on your desktop and choose screen saver options.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.17.c001 | Customize the wallpaper | Current wallpaper thumbnail | other | — | — | Shows the wallpaper currently used on your Mac. | — |
| MD.17.c002 | Customize the wallpaper | Show on all Spaces | toggle | — | — | Use the same wallpaper on every desktop space and display. | Relevant if you created additional desktop spaces on your Mac or connected more than one display. |
| MD.17.c003 | Customize the wallpaper | Screen Saver | other | — | — | Select a screen saver image, choose when the screen saver starts, and choose whether to use automatic or custom settings. | — |
| MD.17.c004 | Customize the wallpaper | Clock Appearance | other | — | — | Choose whether the large clock appears and where it appears: on the Lock Screen, or on both the Screen Saver and Lock Screen. Choose a clock font and weight. | — |
| MD.17.c005 | Customize the wallpaper | Dynamic Wallpapers | popup | Automatic; Dynamic; Light (Still); Dark (Still) | — | Choose an image as your wallpaper. If a pop-up menu appears next to the current wallpaper thumbnail, choose an appearance. Automatic switches from light to dark at sunset based on your current location when Appearance is set to Auto in System Settings > Appearance; when Appearance is set to Light or Dark, the wallpaper remains light or dark. Dynamic gradually brightens and darkens based on the time of day for your current location. Light (Still) displays a static, light version of the picture. Dark (Still) displays a static, dark version of the picture. | The appearance pop-up menu appears only if available. Automatic behavior depends on the Appearance setting and current location. |
| MD.17.c006 | Customize the wallpaper | Your Photos | other | — | — | From photos, folders, and albums that you added, select an image as your wallpaper. | — |
| MD.17.c007 | Customize the wallpaper | Landscape, Cityscape, Underwater, and Earth aerials | other | — | — | Choose an aerial image as your wallpaper. | — |
| MD.17.c008 | Customize the wallpaper | Mac | other | — | — | Choose a color for animated gradient pill shapes that spell “Mac”. | — |
| MD.17.c009 | Customize the wallpaper | Shuffle Aerials | other | — | — | Choose to rotate through several aerial images as your wallpaper. | — |
| MD.17.c010 | Customize the wallpaper | Shuffle | popup | — | — | Choose how often to rotate through the aerials from the pop-up menu at the top of the window. | — |
| MD.17.c011 | Customize the wallpaper | Pictures | other | — | — | Choose an image as your wallpaper. | — |
| MD.17.c012 | Customize the wallpaper | Colors | other | — | — | Choose a color as your wallpaper. | — |

Notes: The supplied page text is incomplete in several places, including the opening navigation instruction and the descriptions for Dynamic Wallpapers, Landscape, Cityscape, Underwater, and Earth aerials, Pictures, and Colors.

#### Independent macOS 27 reports for MD.17

| ID | Kind | Pane | Claim | Source |
|---|---|---|---|---|
| MD.17.x01 | behavior | Wallpaper | The README gives a URL example that opens the `ScreenSaver` anchor using the Wallpaper pane identifier. | T2 [GitHub](https://github.com/jaywcjlove/SystemSettings-URLs-macOS/blob/main/README.md) |
| MD.17.x02 | new | Wallpaper | macOS 27 includes new static and dynamic wallpapers and a new screen saver. | T4 [geeksmodo](https://geeksmodo.com/macos-system-settings/) |
| MD.17.x03 | behavior | Wallpaper | Wallpaper is configured in System Settings > Wallpaper; the article also gives the screen saver path as System Settings > Wallpaper > Screen Saver… | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |

## 13.18 MD.18 Screen Saver

Status: **PARTIAL**. Screen Saver page captured; whether it is a separate sidebar item or reached via Wallpaper is unverified (GAP-B4-09).


### Screen Saver: Change Screen Saver settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp1227/27/mac/27) · topic `mchlp1227` · path: Apple menu > System Settings > Screen Saver · "Open … for me" link: no

Use Screen Saver settings to hide the desktop when you’re away from your Mac or need extra privacy. Choose a screen saver and set when it starts.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.18.c001 | Screen Saver settings | Current screen saver thumbnail | info | — | — | Shows the screen saver currently used on your Mac. | — |
| MD.18.c002 | Screen Saver settings | Preview | button | — | — | Shows a full-screen preview. Move the pointer over the thumbnail at the top of the window, then click Preview. | Move the pointer to stop the preview. |
| MD.18.c003 | Screen Saver settings | Start Screen Saver | popup | — | — | Changes how long your Mac can be inactive before the screen saver starts. | — |
| MD.18.c004 | Screen Saver settings | Use Screen Saver | other | Automatic; Custom | — | Choose Automatic or Custom settings. | — |
| MD.18.c005 | Screen Saver settings | Current screen saver | other | Automatic; Light; Dark | — | Shows the name of the current screen saver and lets you choose options such as Automatic, Light, or Dark. | Options such as Automatic, Light, or Dark are available only if available for the selected screen saver. |
| MD.18.c006 | Screen Saver settings | macOS | other | — | — | Choose a slow-motion image as your screen saver. | — |
| MD.18.c007 | Screen Saver settings | Landscape, Cityscape, Underwater, and Earth aerials | other | Landscape; Cityscape; Underwater; Earth aerials | — | Choose a slow-motion aerial as your screen saver. | — |
| MD.18.c008 | Screen Saver settings | Mac | other | — | — | Choose animated gradient pill shapes that spell “Mac”. | — |
| MD.18.c009 | Screen Saver settings | Shuffle Aerials | other | — | — | Choose to rotate through several slow-motion aerials as your screen saver. | — |
| MD.18.c010 | Screen Saver settings | Shuffle | popup | — | — | Choose how often to rotate through the aerials from the pop-up menu next to Shuffle at the top of the window. | — |
| MD.18.c011 | Screen Saver settings | Other | other | Photos; one of the screen savers; Random | — | Click Photos or one of the screen savers. Click Random to shuffle through the screen savers. | — |
| MD.18.c012 | Screen Saver settings | Options | button | — | — | Set color, speed, and more, based on the screen saver you select. | — |

Notes: The page text provided does not state a macOS version. To start or stop the screen saver by moving the pointer to a screen corner, choose Apple menu > System Settings > Desktop & Dock > Hot Corners.

## 13.19 MD.19 Notifications

Status: **PARTIAL**. Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13).


### Notifications: Change Notifications settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl205da693/27/mac/27) · topic `mchl205da693` · path: System Settings > Notifications · "Open … for me" link: yes

Choose when notifications are displayed and change notification settings for individual apps.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.19.c001 | Notification Center | “Show previews” | popup | Always; When Unlocked; Never | — | Choose when notification previews are shown. | — |
| MD.19.c002 | Notification Center | “when display is sleeping” | toggle | On; Off | — | Choose whether to allow notifications when the display is sleeping. | — |
| MD.19.c003 | Notification Center | “when screen is locked” | toggle | On; Off | — | Choose whether to allow notifications when the screen is locked. | — |
| MD.19.c004 | Notification Center | Notifications Off / Allow Notifications | other | Notifications Off; Allow Notifications | — | Choose whether to allow notifications when mirroring or sharing the display. | — |
| MD.19.c005 | Notification Center | Summarize notifications | toggle | On; Off | — | Choose whether to summarize notifications. | Appears when Apple Intelligence is turned on. Apple Intelligence may not be available in all languages or regions; usage limits may apply. |
| MD.19.c006 | Application Notifications | Application Notifications | list | — | — | Click an app to change its notification settings. | — |
| MD.19.c007 | Application Notifications | “Allow notifications” | toggle | On; Off | — | Turn notifications for the selected app on or off. | — |
| MD.19.c008 | Application Notifications | notification style | other | — | — | If notifications are turned on, select a notification style. | Available for the selected app when notifications are turned on. |

Notes: The page does not state defaults for the controls. The page mentions Apple Intelligence language and region availability, and that usage limits may apply.

#### Independent macOS 27 reports for MD.19

| ID | Kind | Pane | Claim | Source |
|---|---|---|---|---|
| MD.19.x01 | behavior | System Settings > Notifications | Notification summaries can be disabled by turning off “Summarize Notifications.” | T5 [WindowsForum](https://windowsforum.com/news/macos-27-removes-apple-intelligence-master-toggle-wont-free-storage.445490/) |
| MD.19.x02 | behavior | Notifications | Summarize Notifications can be turned off at System Settings > Notifications > Summarize Notifications. | T4 [PCMag UK](https://uk.pcmag.com/macos/167598/is-apple-intelligence-stealing-your-macs-storage-heres-how-to-turn-it-off) |
| MD.19.x03 | behavior | Notifications | Notifications settings are in System Settings > Notifications and allow per-app alert customization, including banners, alerts, silent notifications, and Notification Center inclusion. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |

## 13.20 MD.20 Sound

Status: **PARTIAL**. Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13).


### Sound: Change Sound settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl9777ee30/27/mac/27) · topic `mchl9777ee30` · path: Apple menu > System Settings > Sound · "Open … for me" link: no

Use Sound settings to change sound effects, adjust volume, and choose sound input and output devices.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.20.c001 | Sound Effects | Alert sound | popup | — | — | Choose a sound to play when you receive an alert. The alert plays when you select it so you can hear how it sounds. | The selected alert sound plays when selected. |
| MD.20.c002 | Sound Effects | Play sound effects through | popup | — | — | Choose a sound output device to play sound effects. | — |
| MD.20.c003 | Sound Effects | Alert volume | slider | — | — | Adjust the alert volume. The alert volume is relative to the output volume of your Mac, so if you lower your Mac volume, alert sounds also play more quietly. | — |
| MD.20.c004 | Sound Effects | Play sound on startup | toggle | — | — | Hear the startup chime when you restart your Mac. | When enabled, the startup chime plays when you restart your Mac. |
| MD.20.c005 | Sound Effects | Play user interface sound effects | toggle | — | — | Play sound effects when you perform certain actions, such as dragging an item to the Trash. | — |
| MD.20.c006 | Sound Effects | Play feedback when volume is changed | toggle | — | — | Play a sound when you change the volume so you can hear the new volume. | — |
| MD.20.c007 | Output options | List of sound output devices | list | — | — | Select the device you want to use for sound output. | — |
| MD.20.c008 | Output options | Output volume | slider | — | — | Adjust the output volume. | — |
| MD.20.c009 | Output options | Mute | checkbox | — | — | Stop sound output by selecting the Mute checkbox. | Stops sound output when selected. |
| MD.20.c010 | Output options | Balance | slider | — | — | Adjust the output volume balance between the left and right channels. | — |
| MD.20.c011 | Input options | List of sound input devices | list | — | — | Select the device you want to use for sound input. | — |
| MD.20.c012 | Input options | Input volume | slider | — | — | Adjust the input volume. If you’re recording sound through your computer’s sound port, you can adjust the input volume to compensate for the sound source being too loud or quiet. | The sound-port compensation guidance applies when recording through your computer’s sound port. |
| MD.20.c013 | Input options | Input level meter | info | — | — | See a visual representation of the sound input level. | — |

Notes: Not all options described here appear for all Mac models.

### Sound > Output & Input: Change the sound output settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp2256/27/mac/27) · topic `mchlp2256` · path: Apple menu > System Settings > Sound > Output · "Open … for me" link: yes

Choose a sound output device and adjust output volume, alert volume, mute, and balance. Available options depend on the Mac and connected devices.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.20.c014 | Output | Output devices | list | The computer’s internal speakers; display speakers (if the display has speakers); speakers, headphones, or other equipment plugged into the Mac; devices available wirelessly through AirPlay. For a device plugged into the computer’s sound port, choose “Headphones”. | — | Select the device to use for sound output. | Available devices depend on the Mac and connected devices. |
| MD.20.c015 | Adjust the volume | Output volume | slider | — | — | Adjusts the computer’s output volume. | — |
| MD.20.c016 | Adjust the volume | Sound in the menu bar | other | Turn the volume up or down. | — | If Sound is in the menu bar, it can be used to adjust volume from anywhere. | Only if Sound is in the menu bar. |
| MD.20.c017 | Adjust the volume | Alert volume | slider | — | — | Adjusts the volume for alert sounds. Alert volume is relative to the computer’s volume, so lowering the computer’s volume also makes alert sounds quieter. | — |
| MD.20.c018 | Stop sound output | Mute | checkbox | Selected or not selected. | — | Stops sound output when selected. | — |
| MD.20.c019 | Adjust the balance | Balance | slider | — | — | Adjusts the sound balance. | — |

Notes: Apps with volume controls, such as Music or iMovie, can set volume only as high as or lower than the computer’s output volume; they do not override Sound settings. Depending on the Mac and devices used, other volume options may be available in Audio MIDI Setup. The page also has an “Open Audio MIDI Setup for me” link.

## 13.21 MD.21 Focus

Status: **PARTIAL**. Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13).


### Focus: Change Focus settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlff5da36d/27/mac/27) · topic `mchlff5da36d` · path: Apple menu > System Settings > Focus · "Open … for me" link: no

Use Focus settings to create and manage Focus modes, choose which people or apps can notify you, and set schedules or filters. Focus settings can be shared across Apple devices.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.21.c001 | Focus settings | Focus list | list | Focuses added or created on this Mac and other Apple devices; Do Not Disturb is always available. | — | Click a Focus to change its settings or remove it. | Focuses from other devices appear if “Share across devices” is on. |
| MD.21.c002 | Focus settings | Add Focus | button | Provided Focuses, such as Gaming or Work; Custom. | — | Create a new Focus. For a custom Focus, enter a name, choose a color and icon, then click OK. | — |
| MD.21.c003 | Focus settings | Share across devices | toggle | — | — | Keep Focus settings up to date across Apple devices. | Devices must be signed in to the same Apple Account. |
| MD.21.c004 | Focus settings | Focus status | button | — | — | Open the controls for sharing Focus status with contacts who message you. | — |
| MD.21.c005 | Focus settings | Share Focus status | toggle | — | — | Let contacts who send you a message know that you have silenced notifications. They can choose to notify you anyway if something is important. | — |
| MD.21.c006 | Focus settings | each Focus you want to share your status from | toggle | Focuses available for sharing status. | — | Turn on each Focus from which you want to share your status. | Sharing requires “Share Focus status” to be on. |
| MD.21.c007 | Focus details | *focus name* | button | — | — | Click the icon next to the Focus name to change its name, color, or icon. For a provided Focus, only its color and icon can be changed. The Do Not Disturb Focus cannot have its name, color, or icon changed. A provided Focus’s default color and icon can be restored using the restore control. | Available settings depend on whether the selected Focus is custom, provided, or Do Not Disturb. |
| MD.21.c008 | Focus details | Intelligent Breakthrough & Silencing | toggle | — | — | When on, Apple Intelligence allows important notifications to interrupt you and silences notifications it determines are not important. | Apple Intelligence availability depends on supported languages, regions, and an Apple Intelligence–enabled device. Usage limits may apply. |
| MD.21.c009 | Focus details | Allowed People | other | Selected people. | — | Allow notifications from selected people. | — |
| MD.21.c010 | Focus details | Allowed Apps | other | Selected apps. | — | Allow notifications from selected apps. | — |
| MD.21.c011 | Focus details | Set a Schedule | other | A set time, location, or while using a certain app. | — | Automatically turn on a Focus according to a schedule. | — |
| MD.21.c012 | Focus details | Focus Filters | other | Calendar, Mail, Messages, or Safari. | — | Customize how these apps behave when this Focus is on. | — |
| MD.21.c013 | Focus details | Delete Focus | button | — | — | Remove a Focus. A custom Focus is deleted; a provided Focus is removed from the list but remains available to add again later. | A custom Focus is deleted. |

Notes: The page mentions macOS 27 in the Apple Intelligence note, but does not explicitly state which macOS version the page applies to. Apple Intelligence availability varies by language and region. The page lists supported device languages and notes that some features may not be available in all regions or languages. The page notes that some Apple Intelligence features using server-side models have daily usage limits, which may vary.

### Focus > Set up Focus: Set up a Focus on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl613dc43f/27/mac/27) · topic `mchl613dc43f` · path: System Settings > Focus · "Open … for me" link: yes

Set up a provided or custom Focus, choose which notifications to allow or silence, schedule when a Focus turns on or off, and add filters that customize app behavior.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.21.c014 | Add a Focus | Add Focus | button | Provided Focuses, such as Gaming or Work; Custom | — | Starts adding a provided Focus or creating a custom Focus. | You can create up to ten custom Focuses. |
| MD.21.c015 | Add a Focus | Custom | other | — | — | Starts creating a custom Focus. | — |
| MD.21.c016 | Add a Focus | name | field | — | — | Enter a name for the custom Focus. | — |
| MD.21.c017 | Add a Focus | color | other | — | — | Select a color for the custom Focus. | — |
| MD.21.c018 | Add a Focus | icon | other | — | — | Select an icon for the custom Focus. | — |
| MD.21.c019 | Add a Focus | OK | button | — | — | Confirms creation of a custom Focus. | — |
| MD.21.c020 | Add a Focus | Intelligent Breakthrough & Silencing | toggle | On or off | — | When on, Apple Intelligence allows important notifications to interrupt you and silences notifications it determines are not important. Notifications specifically allowed or silenced are still allowed or silenced. | Apple Intelligence is required for this behavior. For the Reduce Interruptions Focus, this is already on and can’t be turned off. |
| MD.21.c021 | Add a Focus | Do Not Disturb | other | — | — | The page says to turn on this Focus in Control Center to quickly silence all notifications. | — |
| MD.21.c022 | Choose which notifications to allow | Notifications | popup | Allow Some People; Silence Some People | — | Choose whether to allow notifications from selected people or silence notifications from selected people. | — |
| MD.21.c023 | Choose which notifications to allow | Add people | button | — | — | Opens contact selection so you can select one or more people for the chosen Notifications setting. | — |
| MD.21.c024 | Choose which notifications to allow | people | list | One or more contacts | — | Select people to allow or silence. A person can be removed from the list. | — |
| MD.21.c025 | Choose which notifications to allow | Allow calls from | popup | Everybody; allowed people only; only people in your contacts list in Contacts; only people in your Favorites on iPhone | — | Choose whose calls can generate notifications. | The Favorites option refers to Favorites on iPhone. |
| MD.21.c026 | Choose which notifications to allow | Allow repeated calls | toggle | On or off | — | When on, receive notifications from anyone who calls two or more times within three minutes. | — |
| MD.21.c027 | Choose which notifications to allow | Notifications | popup | Allow Some Apps; Silence Some Apps | — | Choose whether to allow notifications from selected apps or silence notifications from selected apps. | — |
| MD.21.c028 | Choose which notifications to allow | Add | button | — | — | Adds the selected apps to the allow or silence list. | — |
| MD.21.c029 | Choose which notifications to allow | apps | list | One or more apps | — | Select apps to allow or silence. An app can be removed from the list. | — |
| MD.21.c030 | Choose which notifications to allow | Time Sensitive Notifications | toggle | On or off | — | When on, allow notifications for events or tasks that require immediate attention. | The page says to also select the option that allows apps to send these notifications in Notifications settings. |
| MD.21.c031 | Choose which notifications to allow | Gaming Focus | info | — | Turns on whenever a game controller is connected with your Mac. | The Gaming Focus is set by default to turn on when a game controller is connected. The page advises specifying people or apps to receive notifications while it is on. | A game controller connected with your Mac. |
| MD.21.c032 | Schedule a Focus to turn on or off automatically | Add Schedule | button | Time; Location; App | — | Adds a time-based, location-based, or app-based schedule. | — |
| MD.21.c033 | Schedule a Focus to turn on or off automatically | Time | other | — | — | Starts setting up a time-based schedule. | — |
| MD.21.c034 | Schedule a Focus to turn on or off automatically | start and end time | field | — | — | Enter the schedule’s start and end times. | — |
| MD.21.c035 | Schedule a Focus to turn on or off automatically | days of the week | other | Select the days the schedule operates | — | Select the days of the week for the time-based schedule. | — |
| MD.21.c036 | Schedule a Focus to turn on or off automatically | Schedule | toggle | On or off | — | Temporarily stops or resumes use of a time-based schedule. | Shown at the top of the window when changing a time-based schedule. |
| MD.21.c037 | Schedule a Focus to turn on or off automatically | Location | other | — | — | Starts setting up a location-based schedule. The Focus turns on when you arrive at the location and turns off when you leave. | Location Services must be enabled in Privacy settings. |
| MD.21.c038 | Schedule a Focus to turn on or off automatically | Search | field | — | — | Enter a place name to find and select a location. | — |
| MD.21.c039 | Schedule a Focus to turn on or off automatically | Automation | toggle | On or off | — | Temporarily stops or resumes use of a location-based or app-based schedule. | Shown at the top of the window when changing a location-based or app-based schedule. |
| MD.21.c040 | Schedule a Focus to turn on or off automatically | App | other | — | — | Starts setting up an app-based schedule. The Focus turns on when you open the app and turns off when you close it or switch to another app. | — |
| MD.21.c041 | Schedule a Focus to turn on or off automatically | Search | field | — | — | Enter an app name to find and select an app. | — |
| MD.21.c042 | Customize app behavior | Add Filter | button | Calendar; Mail; Messages; Safari | — | Adds a Focus filter for a supported app. | — |
| MD.21.c043 | Customize app behavior | Calendar | other | — | — | Opens setup for a Calendar Focus filter. | — |
| MD.21.c044 | Customize app behavior | calendars | list | Select the calendars to see when this Focus is on | — | Select which calendars appear while the Focus is on. | — |
| MD.21.c045 | Customize app behavior | Add | button | — | — | Adds the Calendar Focus filter. | — |
| MD.21.c046 | Customize app behavior | Mail | other | — | — | Opens setup for a Mail Focus filter. | — |
| MD.21.c047 | Customize app behavior | email accounts | list | Select the email accounts to see when this Focus is on | — | Select which email accounts appear while the Focus is on. | — |
| MD.21.c048 | Customize app behavior | Add | button | — | — | Adds the Mail Focus filter. | — |
| MD.21.c049 | Customize app behavior | Messages | other | — | — | Opens setup for a Messages Focus filter. | — |
| MD.21.c050 | Customize app behavior | Filter by People List | toggle | On or off | — | Turn on to filter Messages by the people list. | — |
| MD.21.c051 | Customize app behavior | Add | button | — | — | Adds the Messages Focus filter. | — |
| MD.21.c052 | Customize app behavior | Safari | other | — | — | Opens setup for a Safari Focus filter. | — |
| MD.21.c053 | Customize app behavior | Choose | button | — | — | Opens selection of the Tab Group to use in Safari while the Focus is on. | — |
| MD.21.c054 | Customize app behavior | Tab Group | list | Select the Tab Group to see when this Focus is on | — | Select the Safari Tab Group for the Focus filter. | — |
| MD.21.c055 | Customize app behavior | Open external links in your Focus Tab Group | toggle | On or off | — | Choose whether external links open in the Focus Tab Group. | — |
| MD.21.c056 | Customize app behavior | Add | button | — | — | Adds the Safari Focus filter. | — |

Notes: The page says Focus changes on Mac are reflected on other Apple devices when Focus is kept up to date across devices. The Gaming Focus is set by default to turn on whenever a game controller is connected with your Mac. The page includes Apple Intelligence language, region, device, and usage-limit qualifications. The final instructions for changing or removing a Focus Filter are incomplete in the supplied text.

## 13.22 MD.22 Screen Time

Status: **PARTIAL**. Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13).


### Screen Time > Downtime: Change Downtime settings in Screen Time on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl10157962/27/mac/27) · topic `mchl10157962` · path: Apple menu > System Settings > Screen Time > Downtime · "Open … for me" link: yes

Use Downtime settings to schedule time away from the screen or turn downtime on or off. During downtime, only allowed apps and phone calls are available.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.22.c001 | Downtime | Downtime | toggle | On or off | — | View the current downtime status and turn downtime on or off. | When you turn downtime on, your child is notified that downtime begins in 5 minutes. Turning it off lets your child resume using their device. |
| MD.22.c002 | Downtime | Schedule | popup | Every Day; Custom; Off | — | View the current downtime schedule status and turn the schedule on or off. Choose Every Day and enter a start and end time to use the same schedule each day. Choose Custom to turn downtime on or off and enter start and end times for each day. Choose Off to suspend the downtime schedule. | — |
| MD.22.c003 | Downtime | Start and end times | field | — | — | Enter start and end times for the downtime schedule. For a Custom schedule, set times for each day. | Shown when setting up an Every Day or Custom schedule. |
| MD.22.c004 | Downtime | Downtime on or off for each day | toggle | On or off | — | For a Custom schedule, turn downtime on or off for each day. | Available when Custom is selected for the schedule. |
| MD.22.c005 | Downtime | Block at Downtime | checkbox | — | — | Block a child’s device during downtime. When the device is blocked, the child can ask for more time. | Available only when you’re using a Screen Time passcode. |

Notes: If you don’t see Downtime, you need to turn on Screen Time for the selected family member. During downtime, only apps that you choose to allow and phone calls are available.

### Screen Time > Content & Privacy: Change Content & Privacy Restrictions settings in Screen Time on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/pjnmr56r8tx4/27/mac/27) · topic `pjnmr56r8tx4` · path: Apple menu > System Settings > Screen Time > Content & Privacy · "Open … for me" link: yes

Use Content & Privacy Restrictions in Screen Time to manage content, features, and privacy on your device.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.22.c006 | Content & Privacy Restrictions | Feature Restrictions | other | — | — | Restrict access to Siri and other settings like AirDrop, Camera, Dictation, and more. | Screen Distance applies to an iPhone or iPad with Face ID. |
| MD.22.c007 | Content & Privacy Restrictions | Content Restrictions | popup | Examples given: explicit or clean content in books or movies. The page does not list the full options. | — | Restrict access to certain types of content in first-party apps like the Books app and Music app. Click the pop-up menu next to each option to modify the content you allow. | — |
| MD.22.c008 | Content & Privacy Restrictions | Privacy Restrictions | other | — | — | Prevent changes to settings like Accounts, Face ID & Passwords, Photos, or Bluetooth Sharing. | — |

Notes: The page also refers readers setting up Screen Time or parental controls for a child to Apple Devices for Kids and Families. The page does not state a macOS version, defaults, or effects such as restart or authentication.

### Screen Time > Always Allowed: Change Always Allowed settings in Screen Time on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl83110a3f/27/mac/27) · topic `mchl83110a3f` · path: Apple menu > System Settings > Screen Time > Always Allowed · "Open … for me" link: yes

Choose who a family member can communicate with during downtime and which apps they can use during downtime or after reaching an applicable time limit.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.22.c009 | Always Allowed | During Downtime, Allow communication with | popup | Everyone; Specific contacts | — | Choose who the family member can communicate with during downtime. Everyone allows one-on-one and group conversations with anyone, including unknown numbers. Specific contacts allows conversations only with specific people in the family member’s iCloud contacts. | Available options vary depending on the current settings for allowed communication during screen time in Communication Limits settings. |
| MD.22.c010 | Always Allowed | Edit Contacts | button | — | — | Use this control to specify the contacts allowed when Specific contacts is selected. | — |
| MD.22.c011 | Always Allowed | Contacts | other | — | — | Manage a family member’s iCloud contacts. | Available if you requested permission to manage the family member’s iCloud contacts in Communication Limits settings in Screen Time, and the family member approved the request. |
| MD.22.c012 | Allowed Apps | Allowed Apps | list | — | — | Choose specific apps that can be used during downtime or after the time limit has been reached. | — |
| MD.22.c013 | Allowed Apps | Search | field | — | — | Enter the name of an app to find it in the list. | — |
| MD.22.c014 | Allowed Apps | Sort | popup | Daily Usage; Name; State | — | Change the order of apps in the list. Daily Usage displays apps from most used to least used; Name displays them in alphabetical order; State displays always-allowed apps at the top. | — |

Notes: If Always Allowed isn’t visible, turn on Screen Time for the selected family member. The page does not specify a macOS version. The page does not state any defaults or keyboard shortcuts.

### Screen Time: Change Screen Time settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlca5139d2/27/mac/27) · topic `mchlca5139d2` · path: Apple menu > System Settings > Screen Time · "Open … for me" link: yes

Use Screen Time settings to view and manage family members’ Screen Time, view activity charts, and configure Screen Time features. Available options vary depending on whether you’re signed in to your Apple Account, whether App & Website Activity is on, and whether you’re a parent or guardian of a child in a Family Sharing group.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.22.c015 | Screen Time settings | Family Member | popup | — | — | Choose the family member whose Screen Time settings you want to view and manage. | Available only if you’re a parent or guardian of a child who is part of a Family Sharing group. |
| MD.22.c016 | Screen Time settings | App & Website Activity | link | — | — | View charts that show app usage. Customize the chart to display usage statistics for a specific time period, app, app category, website, or device. | — |
| MD.22.c017 | Screen Time settings | Notifications | link | — | — | View charts showing how many times you receive notifications. Customize the chart to display the number received for a specific time period, app, website, or device. | — |
| MD.22.c018 | Screen Time settings | Pickups | link | — | — | View charts showing how many times you wake your device to use it and the first app you use after waking it. Customize the chart to display pickup counts for a specific time period, app, website, or device. | — |
| MD.22.c019 | Screen Time settings | Downtime | link | Same schedule every day; a different schedule for each day of the week; turn downtime on immediately | — | Schedule time away from the screen. | — |
| MD.22.c020 | Screen Time settings | App Limits | link | — | — | Limit the amount of time spent in apps and on websites. Set limits for specific apps, app categories, and websites. | — |
| MD.22.c021 | Screen Time settings | Always Allowed | link | — | — | Specify apps that can be used at any time, even during downtime. | — |
| MD.22.c022 | Screen Time settings | Screen Distance | link | — | — | Receive an alert when you hold an iPhone or iPad with Face ID too close for an extended period. | Applies to an iPhone or iPad with Face ID. |
| MD.22.c023 | Screen Time settings | Communication Limits | link | — | — | Manage contacts and restrict who you can communicate with during screen time and downtime. | — |
| MD.22.c024 | Screen Time settings | Communication Safety | link | — | — | Check for shared photos or videos sent or received in supported apps and services that may contain nudity or gore. | Nudity feature: macOS 12.1 or later, iOS 15.2 or later, or iPadOS 15.2 or later; not available in all countries or regions. Gore feature: macOS 27 or later. |
| MD.22.c025 | Screen Time settings | Content & Privacy | link | — | — | Restrict content, purchases, downloads, and privacy settings. | — |
| MD.22.c026 | Screen Time settings | Manage Screen Time | link | — | — | Set up a passcode and stay aware of your child’s app and website activity. | Appears only if you choose a child in the Family Member pop-up menu. |
| MD.22.c027 | Screen Time settings | Requests | link | — | — | Respond to requests for more screen time from family members. | — |
| MD.22.c028 | Screen Time settings | Share across devices | toggle | — | — | Include time spent on other devices where you’re signed in to the same Apple Account in Screen Time reports. Turn this on on each device you want included. | Available when you’re signed in to your Apple Account. |
| MD.22.c029 | Screen Time settings | Lock Screen Time Settings | button | — | — | Require a passcode to prevent unauthorized changes to Screen Time settings and to grant more time when limits expire. | If the family member has an administrator account, you’re prompted to convert it to a standard account. For a child selected in the Family Member pop-up menu, this option is in Manage Screen Time settings.; If the family member has an administrator account, you’re prompted to convert it to a standard account. |
| MD.22.c030 | Screen Time settings | App & Website Activity | toggle | Turn on; turn off | — | Turn Screen Time on or off, including features like real-time reporting, scheduled time away from the screen, and time limits for apps. | — |
| MD.22.c031 | Screen Time settings | Screen Time Family Sharing | other | Set up Family Sharing; turn it off | — | Set up Family Sharing, or turn it off to stop sharing your child’s Screen Time. | Available if you haven’t already set up Family Sharing. If you choose a child in the Family Member pop-up menu, this option is in Manage Screen Time settings. |

Notes: The page says available options vary depending on Apple Account sign-in, whether App & Website Activity is on, and whether you’re a parent or guardian of a child in a Family Sharing group. No page-wide macOS version is specified.

### Screen Time > App & Website Activity: View App & Website Activity settings in Screen Time on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchle37ec855/27/mac/27) · topic `mchle37ec855` · path: Apple menu > System Settings > Screen Time > App & Website Activity · "Open … for me" link: yes

Use App & Website Activity in Screen Time to view app-usage statistics for one device or multiple devices, by day or week, and for apps or app categories.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.22.c032 | Device | Device | popup | A device; All Devices | — | Choose which devices’ statistics to display. Choose a device for its statistics, or All Devices for combined statistics. | All Devices is available if you or a family member has Screen Time set up on multiple devices. |
| MD.22.c033 | Date | Today | button | — | — | Show statistics for today. Click Today before choosing This Week to show this week’s statistics. | — |
| MD.22.c034 | Date | Left and right arrow buttons | button | — | — | Change the date. Use the left arrow to choose a previous day or a date in a previous week; the right arrow changes the date in the other direction. | — |
| MD.22.c035 | Date | Date | popup | A day or week, including This Week and Last Week | — | Choose the day or week for which to display statistics. To show last week, first use the left arrow to choose a date in last week, then choose Last Week. | — |
| MD.22.c036 | Chart | Chart | info | — | — | Bars represent total usage. Colored parts represent the top three categories shown under the chart; gray parts represent usage outside those categories. | — |
| MD.22.c037 | Show | Show | popup | Show Apps; Show Categories | — | Choose whether to view statistics for apps or app categories. | — |
| MD.22.c038 | Show | App or category | list | An app or category; All Apps; All Categories; All Usage | — | After choosing Show Apps, select an app, All Apps, or All Usage. After choosing Show Categories, select a category, All Categories, or All Usage. | — |
| MD.22.c039 | Show | Limits column | other | — | — | Hover over a row in the Limits column and click the control shown to set a time limit for an app or category. To change an existing limit, use App Limits settings. | — |
| MD.22.c040 | Show | Limits column information control | other | — | — | Hover over a row in the Limits column and click the information control to view information that may include the app category, age rating, time limit, and time remaining. | Information provided varies by app. |

Notes: The page does not specify a macOS version. The source’s navigation instruction is truncated after “To view app usage on your Mac, choose Apple menu”; the remaining path is identified from the settings pane described.

### Screen Time > App Limits: Change App Limits settings in Screen Time on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchld1c137dd/27/mac/27) · topic `mchld1c137dd` · path: Apple menu > System Settings > Screen Time > App Limits · "Open … for me" link: yes

Use App Limits in Screen Time to create, enable or disable, edit, and remove time limits for apps, categories, and websites.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.22.c041 | App Limits | App Limits | toggle | On or off | — | View the status of app limits, turn them on or off, and view current limits. | If App Limits isn’t visible, Screen Time must be turned on for the selected family member.; When turned off, existing limits are saved but aren’t applied until App Limits is turned on again. |
| MD.22.c042 | Add Limit | Add Limit | button | — | — | Starts creating a limit in the “Create a new app limit” list. You can expand a category to see its apps; search for and select an individual app; select a category or “All Apps & Categories”; or enter a website URL. | — |
| MD.22.c043 | Add Limit | Checkbox next to an app or category | checkbox | Individual apps; categories; “All Apps & Categories” | — | Select an app or category to include it in the new limit. Selecting “All Apps & Categories” applies the limit to all apps and categories. | — |
| MD.22.c044 | Add Limit | Website URL | field | — | — | Enter a website URL to create a limit for that website. The page indicates clicking an icon to add a website, but the icon’s label is not shown. | — |
| MD.22.c045 | Add Limit | Every Day | radio | Every Day; Custom | — | Choose this option to use the same time limit each day, then enter an amount of time. | — |
| MD.22.c046 | Add Limit | Custom | radio | Every Day; Custom | — | Choose this option to use a different time limit for each day of the week. | — |
| MD.22.c047 | Add Limit | Edit | button | — | — | In the Custom schedule, opens the controls for entering an amount of time for each day of the week. | Used after selecting Custom. |
| MD.22.c048 | Add Limit | Amount of time | field | — | — | Enter the allowed time. Use one amount for Every Day, or enter an amount for each day when using Custom. | — |
| MD.22.c049 | Edit | Edit | button | — | — | Hover over a limit in the list and click Edit to change its time limit, change the apps and categories it covers, or remove it. | A limit must already exist in the list. |
| MD.22.c050 | Edit | Done | button | — | — | Finish changing a limit’s time amount. | — |
| MD.22.c051 | Edit | Edit Apps | button | — | — | Opens the app and category selections for a limit; select or deselect checkboxes to change what the limit covers. | Available while editing an existing limit. |
| MD.22.c052 | Edit | Checkbox next to an app or category | checkbox | Apps and categories included in the limit | — | Select or deselect apps and categories to change which items the limit applies to. | Available from Edit Apps. |
| MD.22.c053 | Edit | Remove Limit | button | — | — | Removes the selected limit. The instructions say to click Remove Limit, then click Remove Limit again. | Available while editing an existing limit.; The limit is removed after the second click. |

Notes: The page does not state a macOS version. No keyboard shortcuts, defaults, restart requirements, or authentication prompts are mentioned.

### Screen Time > Communication Limits: Change Communication Limits settings in Screen Time on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlfc8c4a7c/27/mac/27) · topic `mchlfc8c4a7c` · path: Apple menu > System Settings > Screen Time > Communication Limits · "Open … for me" link: yes

Communication Limits in Screen Time restrict who a family member can contact through phone calls, FaceTime, Messages, and iCloud contacts. Communication with known emergency numbers is always allowed.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.22.c054 | Communication Limits | During Screen Time, Allow communication with | popup | Everyone; Contacts only; Contacts and groups with at least one contact | — | Choose who the family member can communicate with during allowed screen time. Everyone allows one-on-one and group conversations with anyone, including unknown numbers. Contacts only allows conversations with people in the family member’s contacts. Contacts and groups with at least one contact allows one-on-one conversations only with contacts and group conversations that include at least one contact. | — |
| MD.22.c055 | Communication Limits | During Downtime, Allow communication with | popup | Contacts only; Specific contacts | — | Choose who the family member can communicate with during downtime. Contacts only allows one-on-one and group conversations with people in the family member’s contacts. Specific contacts allows conversations only with specific people in the family member’s contacts. | Available options vary depending on the current settings for allowed communication during screen time. |
| MD.22.c056 | Communication Limits | Select Contacts | button | — | — | Click to specify the contacts allowed when Specific contacts is selected for downtime communication. | — |
| MD.22.c057 | Communication Limits | Allow contact editing | toggle | On; Off | — | Let your family member edit their contacts in the Contacts app. Turn this option off to prevent them from editing their contacts. | — |
| MD.22.c058 | Communication Limits | Manage [family member’s] iCloud Contacts | toggle | On; Off | — | Request to view and manage your family member’s iCloud contacts. Turning it on sets up iCloud contacts management; turning it off stops managing the family member’s iCloud contacts. | Turning it on sends a request to the family member. |
| MD.22.c059 | Communication Limits | Contacts | info | — | — | Shows the number of the family member’s iCloud contacts and provides access to manage them. | This option appears only after the family member accepts the request to manage their contacts.; Until the request is accepted, Pending is displayed. |
| MD.22.c060 | Communication Limits | Edit | button | — | — | Click to view, edit, add, or remove the family member’s iCloud contacts. | The Edit button appears only after the family member accepts the request to manage their contacts. |

Notes: The available downtime communication options vary depending on the current settings for allowed communication during screen time. Communication to known emergency numbers is always allowed.

#### Independent macOS 27 reports for MD.22

| ID | Kind | Pane | Claim | Source |
|---|---|---|---|---|
| MD.22.x01 | new | Screen Time | Screen Time adds Time Allowances for app categories. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/13/) |
| MD.22.x02 | new | Screen Time | Screen Time adds Schedules for app access during certain times of the day. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/13/) |
| MD.22.x03 | new | Screen Time | Ask to Browse lets children request permission to visit websites. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/13/) |
| MD.22.x04 | changed | Screen Time | Screen Time expands protections against gore and violence alongside existing nudity protections. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/13/) |
| MD.22.x05 | restriction | Screen Time | The new Screen Time features require all devices in a Family Sharing group to be on Apple’s latest series of operating systems. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/13/) |
| MD.22.x06 | behavior | System Settings > Screen Time | For a family member’s device, a parent or guardian in Family Sharing first selects the family member from the pop-up menu, then opens Content & Privacy and Intelligence & Siri. | T5 [WindowsForum](https://windowsforum.com/news/macos-27-removes-apple-intelligence-master-toggle-wont-free-storage.445490/) |
| MD.22.x07 | behavior | System Settings > Screen Time | Screen Time restrictions can be protected with a passcode, which keeps a restricted feature blocked if someone turns its app-level toggle back on. | T5 [WindowsForum](https://windowsforum.com/news/macos-27-removes-apple-intelligence-master-toggle-wont-free-storage.445490/) |
| MD.22.x08 | behavior | Screen Time | On macOS 27, the article says to enable Screen Time, open Content & Privacy, turn its toggle off, then open Siri and select Turn Off Siri. It says Siri can then be changed to Siri Classic. | T4 [PCMag UK](https://uk.pcmag.com/macos/167598/is-apple-intelligence-stealing-your-macs-storage-heres-how-to-turn-it-off) |
| MD.22.x09 | behavior | Screen Time | Screen Time can show app and website usage, set Time Allowances, schedule app availability, restrict content, and generate usage reports. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |

## 13.23 MD.23 Lock Screen

Status: **PARTIAL**. Guide lacks the user-name/photo visibility switch reported by geeksmodo (GAP-B4-08).


### Lock Screen: Change Lock Screen settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mh11784/27/mac/27) · topic `mh11784` · path: Apple menu > System Settings > Lock Screen · "Open … for me" link: no

Use Lock Screen settings to protect your Mac from malware and tampering.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.23.c001 | Lock Screen | Turn display off on battery when inactive | popup | — | — | Choose how long your Mac can be inactive when running on battery power before the screen saver starts. | — |
| MD.23.c002 | Lock Screen | Turn display off on power adapter when inactive | popup | — | — | Choose how long your Mac can be inactive when connected to a power adapter before the screen saver starts. | — |
| MD.23.c003 | Lock Screen | Require password after screen saver begins or display is turned off | popup | Select the amount of time after the screen saver or sleep begins before a password is required. | — | Lock your Mac when it’s left in an idle state. When the Mac wakes, a user login password must be entered before the Mac can be used. | A user login password is required before the Mac can be used after it wakes. |
| MD.23.c004 | Lock Screen | Show password hints | checkbox | — | — | Provide a password hint in the login window. | The hint is shown if the user clicks the question mark in the password field or enters a password incorrectly three times in a row. |
| MD.23.c005 | Lock Screen | Show message when locked | checkbox | — | — | Display a short message in the login window when your Mac is locked. The message is shown to any users who attempt to log in. It can also be used to provide contact information for a lost Mac. | — |
| MD.23.c006 | Lock Screen | Login window shows | popup | List of users; Name and password | — | Select what’s displayed in the login window. “List of users” allows users to choose from a list of user names when logging in. “Name and password” requires users to enter both their user name and password. | — |
| MD.23.c007 | Lock Screen | Show the Sleep, Restart, and Shut Down buttons | checkbox | — | — | Display these buttons in the login window. | — |
| MD.23.c008 | Lock Screen | Accessibility Options | button | — | — | Use accessibility options that make it easier to log in to your Mac. See Turn on accessibility options in the login window. | — |

#### Independent macOS 27 reports for MD.23

| ID | Kind | Pane | Claim | Source |
|---|---|---|---|---|
| MD.23.x01 | new | Lock Screen | A new switch is on by default, preserving the behavior of Tahoe and earlier. Turning it off hides usernames and photos when the Mac is locked. | T4 [geeksmodo](https://geeksmodo.com/macos-system-settings/) |
| MD.23.x02 | renamed | Lock Screen, Touch ID & Password, and Users & Groups | The listed commands open these sections. Touch ID & Password is called Login Password on devices without Touch ID; alternate Touch ID & Password and Users & Groups commands are marked legacy. | T2 [GitHub](https://github.com/bvanpeski/SystemPreferences/blob/main/macos_preferencepanes-Ventura.md) |

## 13.24 MD.24 Privacy & Security

Status: **PARTIAL**. Privacy list captured; "Accessibility" vs "Device Control and Data Access" conflict (GAP-B4-02).


### Privacy & Security > Lockdown Mode: Lock down your Mac if you’re targeted by a cyberattack

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/ibrw66f4e191/27/mac/27) · topic `ibrw66f4e191` · path: Apple menu > System Settings > Privacy & Security · "Open … for me" link: yes

Lockdown Mode is an optional, extreme protection for people who may be personally targeted by highly sophisticated digital threats. Turning it on or off requires restarting the Mac and entering the login password.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.24.c001 | Turn on Lockdown Mode | Open Privacy & Security settings for me | link | — | — | Opens Privacy & Security settings. | — |
| MD.24.c002 | Turn on Lockdown Mode | Turn On | button | — | — | Begins the process of turning on Lockdown Mode. | — |
| MD.24.c003 | Turn on Lockdown Mode | Turn On Lockdown Mode | button | — | — | Confirms turning on Lockdown Mode. | — |
| MD.24.c004 | Turn on Lockdown Mode | Turn On & Restart | button | — | — | Turns on Lockdown Mode and restarts the Mac. | The Mac restarts. Enter the login password, then click OK. |
| MD.24.c005 | Turn on Lockdown Mode | login password | field | — | — | Enter your login password after the Mac restarts. | Authentication is required. |
| MD.24.c006 | Turn on Lockdown Mode | OK | button | — | — | Confirms the entered login password. | — |
| MD.24.c007 | Turn off Lockdown Mode | Open Privacy & Security settings for me | link | — | — | Opens Privacy & Security settings. | — |
| MD.24.c008 | Turn off Lockdown Mode | Turn Off | button | — | — | Begins the process of turning off Lockdown Mode. | — |
| MD.24.c009 | Turn off Lockdown Mode | Turn Off & Restart | button | — | — | Turns off Lockdown Mode and restarts the Mac. | The Mac restarts. Enter the login password, then click OK. |
| MD.24.c010 | Turn off Lockdown Mode | login password | field | — | — | Enter your login password after the Mac restarts. | Authentication is required. |
| MD.24.c011 | Turn off Lockdown Mode | OK | button | — | — | Confirms the entered login password. | — |

Notes: The page does not state a macOS version. After the Mac restarts with Lockdown Mode on, you can turn it off for individual websites in Safari.

### Privacy & Security > Location Services: Control access to the location of your Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mh35873/27/mac/27) · topic `mh35873` · path: Apple menu > System Settings > Privacy & Security > Location Services · "Open … for me" link: yes

Location Services lets apps and websites use information based on your Mac’s current location. You can turn it off globally or manage access for individual apps and system services.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.24.c012 | Turn off Location Services | Open Privacy & Security settings for me | link | — | — | Opens Privacy & Security settings. | — |
| MD.24.c013 | Turn off Location Services | Location Services | toggle | On or off | — | Enables or disables Location Services for the Mac. | When off, your precise location is not sent to Apple. Apple may use your internet connection’s IP address to approximate your location for relevant search suggestions. The Mac may still use location information for emergency calls, and third-party apps and websites may determine your location through other methods. |
| MD.24.c014 | Specify which apps and system services can use Location Services | Open Privacy & Security settings for me | link | — | — | Opens Privacy & Security settings. | — |
| MD.24.c015 | Specify which apps and system services can use Location Services | Location Services for each app | toggle | On or off for each app in the list | — | Controls whether each listed app can use Location Services. | The app must be listed and attempt to use location data.; If turned off for an app, you’re asked to turn it on again the next time that app tries to use your location data. |
| MD.24.c016 | Specify which apps and system services can use Location Services | Details | button | — | — | Appears beside System Services and opens details of specific system services that use your location. | Scroll to the bottom of the app list to reveal System Services. |
| MD.24.c017 | Specify which apps and system services can use Location Services | Suggestions & Search | toggle | On or off | — | Allows your Mac’s location to be used by Siri Suggestions and Safari Suggestions. | When you use Siri Suggestions or Safari Suggestions, your Mac’s location at the time you submit a search query to Safari or Spotlight is sent to Apple to make suggestions more relevant and improve other Apple products and services. |
| MD.24.c018 | Specify which apps and system services can use Location Services | “Significant locations” | toggle | On or off | — | Allows your Mac to identify places significant to you and provide related information in Maps, Calendar, Reminders, and more. | Significant locations are encrypted and can’t be read by Apple. |
| MD.24.c019 | Specify which apps and system services can use Location Services | Details | button | — | — | Opens a list of locations identified as significant. | Associated with “Significant locations”. |
| MD.24.c020 | Specify which apps and system services can use Location Services | Significant locations list | list | Select and remove locations | — | Displays locations that have been identified as significant. | Open Details for “Significant locations”. |
| MD.24.c021 | Specify which apps and system services can use Location Services | An arrow | other | — | — | The page says to click an arrow after describing the significant locations list; the arrow’s purpose or label is not specified in the supplied text. | — |

Notes: The supplied page text does not state a macOS version. The navigation path is inferred from the page’s reference to Privacy & Security settings and Location Services. The page recommends learning about third-party apps’ and websites’ privacy practices; information they collect is governed by their terms and privacy policies.

### Privacy & Security > FileVault: Protect data on your Mac with FileVault

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mh11785/27/mac/27) · topic `mh11785` · path: Apple menu > System Settings > Privacy & Security > FileVault · "Open … for me" link: yes

FileVault protects data on your Mac by requiring a login password to unlock the startup disk. Macs with Apple silicon or a T2 Security Chip encrypt data automatically; turning on FileVault adds protection against access without the login password. Macs without Apple silicon or a T2 chip need FileVault turned on to encrypt data. An administrator is required to set it up.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.24.c022 | Setup instructions — step 2 | FileVault | toggle | On or off | — | Turn on FileVault. | You must be an administrator to set up FileVault. Macs without Apple silicon or a T2 Security Chip need FileVault turned on to encrypt data.; You might be asked to enter your password. |
| MD.24.c023 | Setup instructions — step 3 | Allow my iCloud account to unlock my disk | radio | Choose an iCloud account to unlock the disk and reset the login password. | — | Choose this if you already use iCloud. | For users who already use iCloud. |
| MD.24.c024 | Setup instructions — step 3 | Set up my iCloud account to reset my password | radio | Choose an iCloud account to unlock the disk and reset the login password. | — | Choose this if you don’t already use iCloud. | For users who don’t already use iCloud. |
| MD.24.c025 | Setup instructions — step 3 | Create a recovery key and do not use my iCloud account. | radio | Create a recovery key instead of using an iCloud account. | — | Write down the recovery key and keep it somewhere safe, separate from the encrypted startup disk. | If the Mac is at a business or school, the institution can also set a recovery key to unlock it.; If you forget your login password and can’t reset it, and also forget the recovery key, you won’t be able to log in and your files and settings will be lost forever. |
| MD.24.c026 | Setup instructions — step 4 | Continue | button | — | — | Continue after choosing how to unlock the disk and reset the login password. | — |
| MD.24.c027 | Setup instructions — additional users | Enable Users | button | — | — | Enable a user to unlock the encrypted disk. | If your Mac has additional users and there’s an Enable Users button, enter a user’s login password before they can unlock the encrypted disk.; Requires entering the selected user’s login password. |
| MD.24.c028 | Setup instructions — additional users | user | list | Select a user. | — | Select the user to enable. | Shown when enabling additional users. |
| MD.24.c029 | Setup instructions — additional users | login password | field | — | — | Enter the selected user’s login password. | Required to enable the selected user.; Authentication is required before the user can unlock the encrypted disk. |
| MD.24.c030 | Setup instructions — additional users | OK | button | — | — | Confirm the selected user’s login password. | Shown after selecting a user and entering their login password. |
| MD.24.c031 | Setup instructions — additional users | Continue | button | — | — | Continue after enabling the user. | Used after clicking Enable Users, selecting a user, entering the login password, and clicking OK. |

Notes: The page does not state a macOS version. When FileVault is turned on, information for additional users is also encrypted; users unlock the encrypted disk with their login password. A recovery key is a string of letters and numbers created for you. Keep a copy somewhere other than the encrypted startup disk, and not in the same physical location as your Mac.

### Privacy & Security > Advanced: Change Privacy & Security Advanced settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mh40595/27/mac/27) · topic `mh40595` · path: Apple menu > System Settings > Privacy & Security > Advanced · "Open … for me" link: no

Use Advanced options in Privacy & Security settings to adjust your Mac’s security settings.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.24.c032 | Advanced | Require an administrator password to access system-wide settings | other | — | — | Prevent users from changing locked system settings without an administrator’s password. | — |
| MD.24.c033 | Advanced | Log out automatically after inactivity | other | — | — | Automatically log out users after the Mac is idle. | — |
| MD.24.c034 | Advanced | Log out after | other | A specified number of minutes | — | Automatically log out users after the Mac is idle for the specified number of minutes. | Appears when “Log out automatically after inactivity” is selected. |
| MD.24.c035 | Advanced | Turn on screen saver when login token is removed | other | — | — | Turn on the screen saver on your Mac when a Smart Card containing a login token is removed. | Appears when you insert a Smart Card that has been added to your login keychain. |

Notes: The introductory navigation instruction is incomplete in the provided text. See also: FileVault recovery options; If you can’t turn on FileVault on Mac.

### Privacy & Security > Security apps allowed: Change Privacy & Security settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl211c911f/27/mac/27) · topic `mchl211c911f` · path: Apple menu > System Settings > Privacy & Security · "Open … for me" link: no

Use Privacy & Security settings to manage information your Mac makes available to others across the internet or a network, and to protect encrypted data.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.24.c036 | Privacy | Location Services | toggle | — | — | Allow apps, system services, and websites to gather and use information based on the current location of your Mac to provide location-based services. | — |
| MD.24.c037 | Location Services | Details | button | — | — | Show specific system services that use your location. | — |
| MD.24.c038 | Location Services | Location-Based Suggestions | toggle | — | — | Allow the location of your Mac to be used by Siri Suggestions and Safari Suggestions. | — |
| MD.24.c039 | Privacy | Calendars | toggle | — | — | Allow listed apps that have requested access to access your calendars. | — |
| MD.24.c040 | Privacy | Contacts | toggle | — | — | Allow listed apps that have requested access to access your contacts. | — |
| MD.24.c041 | Privacy | Files & Folders | toggle | — | — | Allow listed apps that have requested access to access files and folders in different locations on this Mac. | — |
| MD.24.c042 | Privacy | Full Disk Access | toggle | — | — | Allow apps to access all files on your computer, including data from other apps (for example, Mail, Messages, Safari, and Home), data from Time Machine backups, and certain administrative settings for all users on this Mac. | — |
| MD.24.c043 | Full Disk Access | (unlabelled) | button | — | — | Click the add button, select an app in the list, then click Open. | — |
| MD.24.c044 | Privacy | HomeKit | toggle | — | — | Allow listed apps that have requested access to access your home data on this Mac. | — |
| MD.24.c045 | Privacy | Media & Apple Music | toggle | — | — | Allow listed apps that have requested access to access Apple Music, your music and video activity, and your media library on this Mac. | — |
| MD.24.c046 | Privacy | Passkeys Access for Web Browsers | toggle | — | — | Allow third-party browsers to use your saved passkeys to sign you in to websites. | — |
| MD.24.c047 | Privacy | Photos | toggle | — | — | Allow listed apps that have requested access to access your photos. Other apps may still have access to items stored outside the Photos Library. | — |
| MD.24.c048 | Privacy | Reminders | toggle | — | — | Allow listed apps that have requested access to access your reminders. | — |
| MD.24.c049 | Privacy | Accessibility | toggle | — | — | Allow apps to run scripts and system commands to control your Mac. Listed apps have requested access. | — |
| MD.24.c050 | Accessibility | (unlabelled) | button | — | — | Click the add button, select an app in the list, then click Open. | — |
| MD.24.c051 | Privacy | App Management | toggle | — | — | Allow apps to update or delete other apps on your Mac. | — |
| MD.24.c052 | Privacy | Automation | toggle | — | — | Allow listed apps that have requested access to access and control other apps on your Mac. | — |
| MD.24.c053 | Privacy | Bluetooth | toggle | — | — | Allow apps to use Bluetooth on this Mac. Listed apps have requested access. | — |
| MD.24.c054 | Bluetooth | (unlabelled) | button | — | — | Click the add button, select an app in the list, then click Open. | — |
| MD.24.c055 | Privacy | Camera | toggle | — | — | Allow apps to access the camera on your Mac. | — |
| MD.24.c056 | Privacy | Developer Tools | toggle | — | — | Allow apps to run software that doesn’t meet the system’s security policy. | — |
| MD.24.c057 | Privacy | Focus | toggle | — | — | Allow apps to see and share your Focus status. | — |
| MD.24.c058 | Privacy | Input Monitoring | toggle | — | — | Allow apps to monitor input from your mouse or trackpad, and to see what you type on the keyboard. Listed apps have requested access. | — |
| MD.24.c059 | Input Monitoring | (unlabelled) | button | — | — | Click the add button, select an app in the list, then click Open. | — |
| MD.24.c060 | Privacy | Local Network | toggle | — | — | Allow apps to find and communicate with devices on your local network. | — |
| MD.24.c061 | Privacy | Microphone | toggle | — | — | Allow apps that have requested access to access the microphone on your Mac. | — |
| MD.24.c062 | Privacy | Motion & Fitness | toggle | — | — | Allow apps to access your motion and fitness activity on your Mac. | — |
| MD.24.c063 | Privacy | Remote Desktop | toggle | — | — | Allow apps to access your screen and audio through Remote Desktop. | — |
| MD.24.c064 | Privacy | Screen & System Audio Recording | toggle | — | — | Allow apps to record your screen and audio, or just your audio. Listed apps have access. | — |
| MD.24.c065 | Screen & System Audio Recording | (unlabelled) | button | — | — | The page says, “To add an app, click” but does not provide further instructions. | — |
| MD.24.c066 | Privacy | Speech Recognition | toggle | — | — | Allow apps that have requested access to access speech recognition on your Mac. | — |
| MD.24.c067 | Privacy | Sensitive Content Warning | toggle | — | — | Avoid seeing photos and videos that may contain sensitive content. Sensitive photos and videos appear blurred; you can choose to view them without blurring. | — |
| MD.24.c068 | Privacy | Blocked Contacts | list | — | — | Add or remove blocked contacts for FaceTime, Messages, and phone calls. | — |
| MD.24.c069 | Privacy | Analytics & Improvements | other | Share Mac Analytics; Improve Siri & Dictation; Improve Assistive Voice Features; Share with app developers; Share iCloud Analytics | — | Manage whether analytics information is sent to Apple, Apple and app developers, or developers, as described by the individual options. | — |
| MD.24.c070 | Analytics & Improvements | Share Mac Analytics | toggle | — | — | Send diagnostics and usage data to Apple. | — |
| MD.24.c071 | Analytics & Improvements | Improve Siri & Dictation | toggle | — | — | Allow Apple to store and review audio of your Siri and dictation interactions. | — |
| MD.24.c072 | Analytics & Improvements | Improve Assistive Voice Features | toggle | — | — | Allow Apple to use audio recordings of your Adaptive Voice Shortcuts and Voice Control interactions. | — |
| MD.24.c073 | Analytics & Improvements | Share with app developers | toggle | — | — | Allow Apple to share data with developers about usage and when an app stops responding. | — |
| MD.24.c074 | Analytics & Improvements | Share iCloud Analytics | toggle | — | — | Allow Apple to use analytics of usage and data from your iCloud account. | — |
| MD.24.c075 | Privacy | Apple Advertising | other | — | — | Review information about personalized ads and Apple’s Advertising and Privacy policy. | — |
| MD.24.c076 | Apple Advertising | Personalized Ads | toggle | — | — | Turn off Personalized Ads to prevent Apple from using your information for ad targeting. This may not decrease the number of ads you receive; they may be less relevant. | The option appears after Apple News or Stocks asks permission to send personalized ads. |
| MD.24.c077 | Apple Advertising | About Apple Advertising & Privacy | link | — | — | Open Apple’s Advertising and Privacy policy. | — |
| MD.24.c078 | Privacy | Apple Intelligence Report | other | Last 15 minutes; 7 days | — | Generate a report of requests Apple Intelligence has sent to Private Cloud Compute during the selected period. | Apple Intelligence requires Siri and the device language to be set to one of the languages listed in the page. Some features and devices may not be available in all regions or languages. Siri AI is described as beta in macOS 27 and requires an Apple Intelligence-enabled device set to a supported language; it is initially available in English. Server-side features may have daily usage limits, which may vary. |
| MD.24.c079 | Security | Allow applications from | popup | App Store; App Store & Known Developers | — | Protect your Mac from malware by allowing apps from the App Store only, or from the App Store and developers identified by Apple. | — |
| MD.24.c080 | Security | Allow accessories to connect | popup | Ask Every Time; Ask for New Accessories; Automatically When Unlocked; Always | — | Control approval for USB or Thunderbolt devices and SD cards connected to your Mac. Ask Every Time requires manual approval each time; Ask for New Accessories requires approval for each new device; Automatically When Unlocked approves a device each time it is connected to your unlocked Mac; Always approves a device each time it is connected. | — |
| MD.24.c081 | Security | FileVault | toggle | Unlock the startup disk with the Apple Account used for iCloud, or with a recovery key created for you | — | Require a login password to access your startup disk. You can add other users who are allowed to unlock and access information on your Mac. | When encryption is turned on, all users are required to use a login password.; When first turning on FileVault, choose the Apple Account or recovery-key method for unlocking the startup disk if you forget your login password. If you choose a recovery key, you must write it down and keep track of it yourself. |
| MD.24.c082 | Security | Lockdown Mode | other | — | — | Limit apps, websites, and features if you believe you’re being personally targeted by a highly sophisticated cyberattack. The page notes that most people are never targeted by attacks of this nature. | — |

Notes: The supplied page text does not state a macOS version that the page applies to; macOS 27 appears only in the Apple Intelligence availability note. The navigation instruction in the supplied text is truncated after “To change these settings, choose Apple menu”; the path is completed using the described System Settings pane. No defaults, restart requirements, confirmation dialogs, or authentication prompts are stated in the supplied text.

### Privacy & Security > Files and folders: Control access to files and folders on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchld5a35146/27/mac/27) · topic `mchld5a35146` · path: Apple menu > System Settings > Privacy & Security > Files & Folders · "Open … for me" link: yes

Choose which apps and websites can access files and folders in specific locations, including Desktop, Downloads, and Documents.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.24.c083 | Files & Folders | the ability to access files and folders in specific locations | toggle | On or off | — | For each app in the list, turn access to files and folders in specific locations on or off. | — |

Notes: The page recommends reviewing the privacy practices of third-party apps or websites allowed to access files and folders; information they collect is governed by their terms and privacy policies. System administrators can refer to Apple Platform Deployment for information about deploying security settings.

#### Independent macOS 27 reports for MD.24

| ID | Kind | Pane | Claim | Source |
|---|---|---|---|---|
| MD.24.x01 | identifier | Privacy & Security | The link ending in `com.apple.preference.security?Privacy_ScreenCapture` opens Privacy & Security at Screen & System Audio Recording. | T4 [Unpixel](https://unpixel.tech/blog/url-schemes-mac/) |
| MD.24.x02 | identifier | Security & Privacy | The README says the AppleScript may not return every deep-linkable anchor; it gives `ListenEvent` as a likely key for the Input Monitoring section in `PrivacyTCCServices.plist`. | T2 [GitHub](https://github.com/jaywcjlove/SystemSettings-URLs-macOS/blob/main/README.md) |
| MD.24.x03 | identifier | Privacy; Accessibility | Some larger sections retain the previous `.preference` schema, particularly Privacy and Accessibility. | T4 [macOS Adventures](https://www.macosadventures.com/2022/12/05/how-to-open-every-section-of-macos-ventura-system-settings/) |
| MD.24.x04 | renamed | System Settings > Privacy & Security | The Accessibility pane was renamed to Device Control and Data Access. | T4 [Michael Tsai](https://mjtsai.com/blog/2026/09/14/macos-27-golden-gate/) |
| MD.24.x05 | renamed | Privacy & Security | On macOS 27, the permission pane Sorla needs is called “Device Control and Data Access” in English and “Enhetskontroll och dataåtkomst” in Swedish, instead of “Accessibility” / “Hjälpmedel.” The issue says this is confirmed by the SecurityPrivacyExtension localized strings for key ACCESSIBILITY on macOS 27.0 (26A428). | T2 [GitHub](https://github.com/markstrom/sorla/issues/74) |
| MD.24.x06 | renamed | Privacy & Security | The requested naming by running macOS version is “Accessibility” / “Hjälpmedel” on macOS 14–26 and “Device Control and Data Access” / “Enhetskontroll och dataåtkomst” on macOS 27+. | T2 [GitHub](https://github.com/markstrom/sorla/issues/74) |
| MD.24.x07 | behavior | Privacy & Security | The deep link x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility opens the pane directly and still works on macOS 27. | T2 [GitHub](https://github.com/markstrom/sorla/issues/74) |
| MD.24.x08 | behavior | System Settings > Privacy & Security | An Apple Community author suggests keeping the Apple Intelligence report setting turned on to check whether Apple Intelligence is functioning. | T5 [WindowsForum](https://windowsforum.com/news/macos-27-removes-apple-intelligence-master-toggle-wont-free-storage.445490/) |
| MD.24.x09 | identifier | Privacy & Security | The listed commands open Privacy & Security. Legacy privacy-pane deep links are supplied for Location Services, Contacts, Calendars, Reminders, Photos, Bluetooth, Microphone, Camera, Motion & Fitness, HomeKit, Speech Recognition, Media & Apple Music, Files and Folders, Full Disk Access, Focus, Accessibility, Input Monitoring, Screen Recording, Passkeys Access for Web Browsers, Automation, App Management, Developer Tools, Sensitive Content Warning, Analytics & Improvements, Apple Advertising, FileVault, Lockdown Mode, and the Advanced button. HomeKit and later designated Sonoma entries are marked macOS 14 Sonoma. Screen Recording is called Screen & System Audio Recording in macOS 14 Sonoma. Extensions and Profiles commands are also listed, including a legacy Profiles alternative. | T2 [GitHub](https://github.com/bvanpeski/SystemPreferences/blob/main/macos_preferencepanes-Ventura.md) |
| MD.24.x10 | removed | Privacy permissions; specific pane not stated | Older PPPC profile keys are being phased out, and Accessibility permissions granted through the old profile format are removed. | T4 [Dr Logic](https://drlogic.com/article/macos-27-is-live-the-security-changes-that-will-quietly-break-on-your-mac-fleet/) |
| MD.24.x11 | behavior | Privacy permissions; specific pane not stated | Users can switch off Accessibility permissions that device management granted automatically. | T4 [Dr Logic](https://drlogic.com/article/macos-27-is-live-the-security-changes-that-will-quietly-break-on-your-mac-fleet/) |
| MD.24.x12 | new | Privacy & Security > Local Network | In macOS 27.2, users can add and remove apps from the Local Network list. | T4 [The Eclectic Light Company](https://eclecticlight.co/2026/10/22/local-network-privacy-is-getting-better/) |
| MD.24.x13 | behavior | Privacy & Security > Local Network | Users can control whether listed apps are allowed or denied local network access. | T4 [The Eclectic Light Company](https://eclecticlight.co/2026/10/22/local-network-privacy-is-getting-better/) |
| MD.24.x14 | behavior | System Settings → Privacy & Security → Analytics & Improvements | Turning off Improve Siri & Dictation stops Apple storing audio and transcripts of Siri and Dictation requests for quality review. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.24.x15 | behavior | System Settings → Privacy & Security → Analytics & Improvements | On Mac, turn off Improve Siri & Dictation; if the Improve Siri & Apple Intelligence toggle is present, turn it off too. The text says iOS 27 adds the latter toggle for model training. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.24.x16 | behavior | System Settings → Privacy & Security → Analytics & Improvements | Turning off Share Mac Analytics stops daily diagnostics and usage data being sent to Apple, including hardware configuration, OS performance, and app usage patterns. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.24.x17 | behavior | System Settings → Privacy & Security → Analytics & Improvements | Turning off Share with App Developers stops third-party app crash data and usage statistics being sent to developers through Apple. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.24.x18 | renamed | System Settings → Privacy & Security | The pane formerly called Accessibility is now called Device Control and Data Access. Apps listed there can control the Mac and read what is on screen. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.24.x19 | behavior | System Settings → Privacy & Security → Device Control and Data Access | Review the listed apps and remove ones you do not recognize; this access is described as among the broadest an app can have. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.24.x20 | behavior | System Settings → Privacy & Security | Golden Gate denies apps' requests to read another developer's app data folders by default, without showing a permission prompt. Access can be granted in Privacy & Security. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.24.x21 | behavior | System Settings → Privacy & Security | Review app permissions for Camera, Microphone, Files and Folders, Full Disk Access, and Screen & System Audio Recording; access can be revoked. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.24.x22 | behavior | System Settings → Privacy & Security → Location Services | Location Services can remain on while per-app access is reviewed. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.24.x23 | behavior | System Settings → Privacy & Security → Location Services → System Services → Details | Turning off Significant Locations stops the Mac logging frequently visited places; turning off Mac Analytics keeps location data out of analytics reports sent to Apple. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.24.x24 | behavior | System Settings → Privacy & Security → Apple Advertising | Turning off Personalized Ads means ads in the App Store, Apple News, and Stocks are not based on interests, search behavior, or app downloads; it does not reduce the number of ads. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.24.x25 | behavior | System Settings → Privacy & Security → FileVault | FileVault encrypts the startup disk and is turned on automatically during setup with an Apple Account. Its recovery key is stored in the end-to-end encrypted Passwords app. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.24.x26 | behavior | System Settings → Privacy & Security → Background Security Improvements | Automatically installing Background Security Improvements lets Apple deliver small security fixes for Safari, WebKit, and other system components between full macOS updates. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.24.x27 | behavior | Safari → Settings → Privacy | Prevent cross-site tracking is on by default and blocks third-party cookies and cross-site tracking using Intelligent Tracking Prevention. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.24.x28 | behavior | Safari → Settings → Privacy | Hide IP address from trackers can be set to Trackers Only, or to Trackers and Websites with iCloud+. The broader option uses Private Relay; without iCloud+, the feature relies on local processing. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.24.x29 | changed | System Settings → Privacy & Security | The default behavior for access to other developers' app data changed: requests that previously triggered a permission prompt are now denied, and access can be allowed in Privacy & Security. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.24.x30 | behavior | Privacy & Security | The article recommends reviewing app permissions in Privacy & Security. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |

## 13.25 MD.25 Touch ID & Password / Login Password

Status: **PARTIAL**. Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13).


### Login Password: Change Login Password settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl0daacc9f/27/mac/27) · topic `mchl0daacc9f` · path: Apple menu · "Open … for me" link: no

Use Login Password settings to change the password you use to log in to your Mac.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.25.c001 | Login Password | A login password has been set for this user | info | — | — | Click Change to change the login password and password hint for the current user. While changing the password, you can use the Password Assistant to help you select a secure password. | — |
| MD.25.c002 | Login Password | Change | button | — | — | Changes the login password and password hint for the current user. | — |
| MD.25.c003 | Login Password | Tips for creating secure passwords | link | — | — | Opens tips for creating secure passwords. | — |

Notes: The provided page text does not specify a macOS version or the rest of the navigation path after “Apple menu”.

### Touch ID & Password: Touch ID & Password settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl319754b3/27/mac/27) · topic `mchl319754b3` · path: Apple menu > System Settings > Touch ID & Password · "Open … for me" link: yes

Use Touch ID & Password settings to change the current user’s login password and, if the Mac or Magic Keyboard has Touch ID, add a fingerprint and configure Touch ID features.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.25.c004 | — | A login password has been set for this user | other | — | — | Change the login password and password hint for the current user. | — |
| MD.25.c005 | — | Add Fingerprint | button | — | — | Add a fingerprint to the current user account. To rename a fingerprint, click the text below the fingerprint, then enter a new name. You can add up to three fingerprints per user account; your Mac can store up to five fingerprints total. | The Mac or Magic Keyboard must have Touch ID. |
| MD.25.c006 | — | Use Touch ID to unlock your Mac | toggle | — | — | Allow Touch ID to unlock your Mac with any fingerprint added to the current user account. | The Mac or Magic Keyboard must have Touch ID.; For security, you need to enter your password when you start up your Mac. |
| MD.25.c007 | — | Use Touch ID for Apple Pay | toggle | — | — | Allow Touch ID to approve Apple Pay purchases. | The Mac or Magic Keyboard must have Touch ID. |
| MD.25.c008 | — | Use Touch ID for purchases in iTunes Store, App Store, and Apple Books | toggle | — | — | Allow Touch ID to make purchases. | The Mac or Magic Keyboard must have Touch ID. |
| MD.25.c009 | — | Use Touch ID for autofilling passwords | toggle | — | — | Allow Touch ID to fill in user names and passwords in Safari and other apps. | The Mac or Magic Keyboard must have Touch ID. |
| MD.25.c010 | — | Use Touch ID for fast user switching | toggle | — | — | Allow Touch ID to switch between Mac user accounts. | The Mac or Magic Keyboard must have Touch ID. |
| MD.25.c011 | — | Use Apple Watch to unlock your apps and your Mac | toggle | — | — | Allow your Apple Watch to unlock apps when they require a password and unlock your Mac when you wake it from an idle state. | Available only if your Mac supports Auto Unlock. |

Notes: No macOS version is stated on the provided page. The page also links to guidance about using Touch ID, resetting a Mac login password, and troubleshooting Touch ID.

## 13.26 MD.26 Users & Groups

Status: **PARTIAL**. Users & Groups captured; legacy login-options topic redirects (GAP-B4-12).


### Users & Groups > User: Change User settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mh11774/27/mac/27) · topic `mh11774` · path: Apple menu > System Settings > Users & Groups · "Open … for me" link: yes

Change settings for your Mac user account in User Settings. Administrators can also change settings for other users of the Mac.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.26.c001 | User account | User picture | button | Photo from photo library; take a new photo using the Mac camera; Memoji; emoji; initials in a monogram; other images | — | Click the picture next to a user name in the login window to choose a login picture. The selected image is also displayed for the user’s Apple Account picture and My Card in Contacts. | Changing the picture also changes the displayed Apple Account picture and My Card image. |
| MD.26.c002 | User account | Change or Reset | button | — | — | Change the login password and password hint for the currently logged-in user, or reset the password for someone else’s account. | — |
| MD.26.c003 | User account | Allow this user to administer this computer | checkbox | — | — | Allow the user to be an administrator for this Mac. | — |
| MD.26.c004 | User account | Allow user to reset password using Apple Account | checkbox | — | — | Allow a user to change their Mac password at login by entering their Apple Account and password. | The user must have set up iCloud on this Mac. This option isn’t available if FileVault is turned on and set to allow the user to reset their password at startup using their Apple Account. |
| MD.26.c005 | User account | Open Contact Card | button | — | — | View and edit a user’s Contacts information. | Only available for the user who is currently logged in. |
| MD.26.c006 | User account | Delete User | button | — | — | Delete a user. | Doesn’t appear if the user is currently logged in or if there is only one user set up. You may need to enter an administrator password.; May require an administrator password. |

Notes: The page does not state a macOS version. Administrators can change settings for other users. The page links to password help and to guidance about changing the macOS user account name and home folder, file sharing, screen sharing, and related settings.

### Users & Groups: Change Users & Groups settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mtusr001/27/mac/27) · topic `mtusr001` · path: Apple menu > System Settings > Users & Groups · "Open … for me" link: yes

Use Users & Groups to manage user accounts and groups, choose automatic login, and manage network account access. You must be an administrator to perform these tasks. Both standard users and administrators can join network account servers.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.26.c007 | Users | Users | list | List of users | — | View the list of users. | — |
| MD.26.c008 | Users | user picture | button | — | — | Click the user picture to change it. | — |
| MD.26.c009 | Users | Change User settings | link | — | — | Open settings for a user. | — |
| MD.26.c010 | Users | Change Sharing Only User settings | link | — | — | Open settings for a Sharing Only user. | — |
| MD.26.c011 | Users | Guest User | toggle | On or off | — | Turn on the Guest User account to allow occasional users to log in. | — |
| MD.26.c012 | Groups | Groups | list | List of groups | — | View the list of groups. | — |
| MD.26.c013 | Groups | Change Group settings | link | — | — | Open group settings. | — |
| MD.26.c014 | Add User | Add User | button | Administrator; Standard; Sharing Only | — | Create a new user, then choose an account type. Administrators can add and manage users, install apps, and change settings. Standard users can install apps and change their own settings, but can’t add users or change other users’ settings. Sharing-only users can access shared files remotely, but can’t log in to the computer or change settings. | Administrator required to perform these tasks.; You may be asked to enter your password. A Sharing Only user may need additional permissions configured in File Sharing, Screen Sharing, or Remote Management settings to access shared files or screens. |
| MD.26.c015 | Add Group | Add Group | button | — | — | Create a new group. Groups allow multiple users to have the same access privileges; control access to shared folders using permissions in File Sharing options in Sharing settings. | Administrator required to perform these tasks.; You may be asked to enter your password. |
| MD.26.c016 | Automatically log in as | Automatically log in as | popup | A user; Off | — | Choose a user and enter the password to enable automatic login, or choose Off to turn it off. With Off, startup opens a login window showing all users. | Automatic login is disabled when FileVault is turned on. The page advises not to use automatic login for an administrator.; When enabled, automatic login takes effect the next time you restart the Mac and logs that user in whenever the Mac starts up. Enabling it allows anyone to access the Mac by restarting it. Follow the onscreen instructions after entering the password. |
| MD.26.c017 | Allow network users to log in at login window | Allow network users to log in at login window | checkbox | Specify which network users can log in | — | Specify which network users can log in to your Mac. | This option only appears if your Mac is joined to a network account server. |
| MD.26.c018 | Network account server | Network account server | button | — | — | Join a network account server, or manage network account server settings. | Both standard users and administrators can join network account servers. |
| MD.26.c019 | Related settings | Open Lock Screen settings for me | link | — | — | Open Lock Screen settings. The page says login-window options such as password hints, the user list, and the Sleep, Restart, and Shut Down buttons can be customized there. | — |

Notes: The page does not state a macOS version. The page says an administrator is required to perform the account and group tasks described. The first user created when setting up a Mac is an administrator. A Mac can have multiple administrators, and standard users can be converted to administrators. The page cautions against sharing administrator names and passwords. With automatic login enabled, the page warns that anyone can access the Mac simply by restarting it. The page also includes an “Open Users & Groups settings for me” link.

#### Users & Groups > Login options — topic `mtusr005` redirects to the guide home

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mtusr005/27/mac/27). NOT FOUND for macOS 27.


## 13.27 MD.27 Internet Accounts

Status: **PARTIAL**. Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13).


### Internet Accounts: Change Internet Accounts settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mh35705/27/mac/27) · topic `mh35705` · path: Apple menu · "Open … for me" link: yes

Use Internet Accounts settings to add accounts from providers such as Google and Yahoo, and choose which account features to use with Mac apps. Turning off features or deleting an account can remove data stored in apps; the data may be restored if the feature is turned on again or the account is added again.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.27.c001 | Internet Accounts | List of accounts | list | — | — | Click an account to view and change the features and settings set up on your Mac for that account. | — |
| MD.27.c002 | Internet Accounts | Details | button | — | — | Update the description and name associated with the account. | Available after clicking a specific account in the list of accounts set up on your Mac. |
| MD.27.c003 | Internet Accounts | List of account features | toggle | Mail, Contacts, Calendar, and more | — | Turn the features you want to use for the account on or off. | Available after clicking a specific account in the list of accounts set up on your Mac.; Turning off a feature can remove data stored in apps. The data may be restored if the feature is turned on again or the account is added again. |
| MD.27.c004 | Internet Accounts | Add Account | button | Account providers such as Google and Yahoo; Add Other Account | — | Add an account from a provider by entering the requested information. For a provider that is not listed, use Add Other Account to choose the type of account and enter its settings. | If you do not have an account with a provider, create one on the provider’s website first. For an unlisted provider, such as a work or school mail or calendar account, the account type and settings may be needed from the provider. |
| MD.27.c005 | Internet Accounts | Add Other Account | button | Types of accounts | — | Choose the type of account to add, then enter the requested account settings. | Used for an account from a provider that is not listed. Ask the provider if you do not know the account type or settings. |
| MD.27.c006 | Internet Accounts | Delete Account | button | — | — | Remove the account and turn off all its features. | Available after clicking a specific account in the list of accounts set up on your Mac.; Deleting an account can remove data stored in apps. The data may be restored if the account is added again. |

Notes: The supplied page text says “To view these settings, choose Apple menu” but does not include the rest of the navigation path. No macOS version, defaults, restart requirements, confirmation prompts, or authentication requirements are stated.

## 13.28 MD.28 Game Center

Status: **PARTIAL**. Game Center settings now documented in the Games User Guide (1.0 / macOS 27), not the Mac User Guide.


#### Game Center — topic `mchle962c097` redirects to the guide home

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchle962c097/27/mac/27). NOT FOUND for macOS 27.


### Game Center: Change Game Center settings on Mac

T1 [Games User Guide](https://support.apple.com/en-bw/guide/games/mchle962c097/1.0/mac/27) · topic `mchle962c097` · path: System Settings > Game Center · "Open … for me" link: yes

Game Center lets you connect with friends, track and compare scores and achievements, compete in challenges, and invite friends to multiplayer games.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.28.c001 | Game Center | Show Profile | Button or navigation item | — | — | View your Game Center profile in the Apple Games app. | — |
| MD.28.c002 | Game Center | Customize Profile | Button or navigation item | — | — | Change your Game Center nickname or avatar. | — |
| MD.28.c003 | Game Center | All Friends | Button or navigation item | — | — | See a list of your Game Center friends, and invite someone to be a Game Center friend. | — |
| MD.28.c004 | Game Center | Friend Requests | Button or navigation item | — | — | See a list of your active friend requests, limit friend requests to your contacts, and invite someone to be a Game Center friend. | — |
| MD.28.c005 | Game Center | Invite Friends | Button or navigation item | — | — | Invite someone to be a Game Center friend. | — |
| MD.28.c006 | Game Center | Share Friends List | Toggle | — | — | Allow apps to access your list of Game Center friends. | — |
| MD.28.c007 | Game Center | Show Apps | Button or navigation item | — | — | See a list of apps that have requested access to your Game Center friends list. | Available when Share Friends List is turned on. |
| MD.28.c008 | Game Center | Help Friends Find You | Toggle | — | — | Allow your Game Center friends to identify you more easily by the email address and phone number associated with your Apple Account. | — |
| MD.28.c009 | Game Center | Activity Sharing | Selection control | Everyone; Friends Only; Only You | — | Specify who can see your profile, achievement progress, leaderboard updates, and recently played games. | Age restrictions vary by country or region. In the United States, options are limited to Friends Only and Only You for children under age 13. |
| MD.28.c010 | Game Center | Nearby Players | Toggle | — | — | Allow nearby Game Center players in the same game to invite you to a multiplayer game over Wi-Fi or Bluetooth. | — |
| MD.28.c011 | Game Center | Identifier | Button or action | — | — | Reset the identifier used to report Game Center usage statistics to Apple. | — |
| MD.28.c012 | Game Center | Sign Out | Button or action | — | — | Sign out of Game Center. | — |

Notes: The source does not state defaults, restart requirements, or confirmation dialogs. Control types are described generically where the source does not specify the exact UI control.

## 13.29 MD.29 Wallet & Apple Pay

Status: **PARTIAL**. Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13).


### Wallet & Apple Pay: Change Wallet & Apple Pay settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl16329b92/27/mac/27) · topic `mchl16329b92` · path: Apple menu > System Settings > Wallet & Apple Pay · "Open … for me" link: yes

Use Wallet & Apple Pay settings to add and manage cards for Apple Pay on websites that support it. The page says this applies when your Mac or Magic Keyboard has Touch ID.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.29.c001 | Options | Add Card | button | Add credit cards, debit cards, prepaid cards, store cards, and Apple Card; use the computer’s camera or type the card information. | — | Click Add Card, then follow the onscreen instructions. | The introductory text applies to a Mac or Magic Keyboard with Touch ID. If you’re not signed in to your Apple Account, you’re asked to sign in. |
| MD.29.c002 | Options | AutoFill Cards | other | Credit, debit, prepaid, or store cards. | — | Choose which cards are enabled for AutoFill. | — |
| MD.29.c003 | Options | Default Card | other | Credit, debit, prepaid, or store cards; Apple Card. | — | Choose the card used by Apple Pay. You can choose a different card when you make a purchase. | — |
| MD.29.c004 | Options | Shipping Address | field | — | — | Enter the shipping address to use with Apple Pay. | — |
| MD.29.c005 | Options | Email | field | — | — | Enter the email address to use with Apple Pay. | — |
| MD.29.c006 | Options | Hide My Email | other | — | — | Send and receive messages related to purchases without having to share your real email address. A random email address is created for each website and app, and messages are forwarded to your real email address. | — |
| MD.29.c007 | Options | Phone | field | — | — | Enter the phone number to use with Apple Pay. | — |
| MD.29.c008 | Options | Compatible Cards | toggle | — | — | Turn on this option to check that cards saved in Safari AutoFill work with Apple Pay and to use those cards in Wallet. | Applies to cards saved in Safari AutoFill. |
| MD.29.c009 | Options | Add Orders to Wallet | other | — | — | See Apple Pay orders made on your Mac on your iPhone. | Orders must be from participating merchants; viewing them requires an iPhone. |
| MD.29.c010 | Options | See how your data is managed | link | — | — | View the Apple Pay Privacy Policy. | — |
| MD.29.c011 | Card management | Remove Card | button | — | — | Click a card, then click Remove Card. | — |
| MD.29.c012 | Card management | Card | other | — | — | Click a card to see additional details, such as the billing address, Device Account Number, or recent transactions on this Mac. | Recent transactions aren’t available for all cards. |

Notes: The page does not state a macOS version. A Device Account Number is unique to your Mac and the selected card. It is generated by your bank or the bank’s authorized service provider when you add a card to Wallet & Apple Pay, and is used instead of your actual card number for greater security. The navigation path is the standard path for the named System Settings pane; the page excerpt itself does not show the full path.

## 13.30 MD.30 Keyboard

Status: **PARTIAL**. Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13).


### Keyboard > Shortcuts: Use macOS keyboard shortcuts

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp2262/27/mac/27) · topic `mchlp2262` · path: Apple menu > System Settings > Keyboard > Keyboard Shortcuts · "Open … for me" link: yes

Keyboard shortcuts perform tasks using modifier-key combinations. You can customize or disable some shortcuts in Keyboard settings.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.30.c001 | Customize keyboard shortcuts | Mission Control | other | — | — | Select this category in the sidebar to view and customize its keyboard shortcuts. | — |
| MD.30.c002 | Customize keyboard shortcuts | Spotlight | other | — | — | Select this category in the sidebar to view and customize its keyboard shortcuts. | — |
| MD.30.c003 | Customize keyboard shortcuts | App Shortcuts | other | — | — | Select this category in the sidebar to customize keyboard shortcuts for specific apps. | — |
| MD.30.c004 | Customize keyboard shortcuts | shortcut checkbox | checkbox | Selected or deselected | — | Select the checkbox next to a shortcut to enable it. Double-click its current key combination to enter a replacement. | A key type, such as a letter key or a function key, can be used only once in a key combination.; Quit and reopen apps in use for a changed shortcut to take effect. A shortcut that conflicts with another command or app shortcut will not work until the conflict is resolved. |
| MD.30.c005 | Customize keyboard shortcuts | current key combination | field | — | — | Double-click the current key combination, then press the new combination. | A key type, such as a letter key or a function key, can be used only once in a key combination.; Quit and reopen apps in use for the new shortcut to take effect. A shortcut that conflicts with another command or app shortcut will not work until the conflict is resolved. |
| MD.30.c006 | Customize keyboard shortcuts | Restore Defaults | button | — | — | Restore all keyboard shortcuts to their original key combinations. | Available in Keyboard settings under Keyboard Shortcuts. |
| MD.30.c007 | Disable a keyboard shortcut | shortcut checkbox | checkbox | Selected or deselected | — | Deselect the checkbox next to a shortcut to disable it. | This is suggested when an app shortcut conflicts with a macOS shortcut. |

Shortcuts stated: Command-N; Command-V

Notes: Keyboard shortcuts in apps may vary depending on the language and keyboard layout used on the Mac. If a shortcut does not work as expected, check the app menus in the menu bar or use Keyboard Viewer to see the current keyboard layout (input source). The page does not state a macOS version or default key combinations.

### Keyboard > Dictation: Dictate messages and documents on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mh40584/27/mac/27) · topic `mh40584` · path: Apple menu > System Settings > Keyboard > Dictation · "Open … for me" link: yes

Use Dictation to enter text by speaking in apps where you can type. Dictation availability and features vary by language, region, and Mac. When Voice Control is on, standard macOS Dictation isn’t available.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.30.c008 | Turn Dictation on or off | Dictation | toggle | On; off | — | Turns Dictation on or off. | Dictation is not available in all languages or regions. |
| MD.30.c009 | Turn Dictation on or off | Enable | button | — | — | Click after turning on Dictation. | May be followed by a prompt asking whether to improve Siri and Dictation. |
| MD.30.c010 | Turn Dictation on or off | Share Audio Recordings | button | — | — | Allows Apple to store audio of Siri and Dictation interactions from the Mac. | Presented if asked whether to improve Siri and Dictation.; Apple may review a sample of stored audio. |
| MD.30.c011 | Turn Dictation on or off | Not Now | button | — | — | Choose not to share audio recordings when asked whether to improve Siri and Dictation. | Presented if asked whether to improve Siri and Dictation. |
| MD.30.c012 | Turn Dictation on or off | Change Privacy & Security settings | link | — | — | Use this to change whether audio recordings are shared. | — |
| MD.30.c013 | Turn Dictation on or off | About Ask Siri, Dictation & Privacy | link | — | — | Opens information about how Apple protects information and lets you choose what you share. | — |
| MD.30.c014 | Dictate text | Automatic insertion of commas, periods, and question marks | other | On; off | — | In supported languages, Dictation automatically inserts commas, periods, and question marks. The page says this feature can be turned off in Keyboard settings but does not provide its exact setting label. | Supported languages. |
| MD.30.c015 | Set the Dictation keyboard shortcut | Shortcut | popup | Available shortcuts; Customize | — | Choose a shortcut to start Dictation. Choosing Customize lets you press the keys you want to use. | The Microphone key and shortcut behavior may depend on the Mac model. |
| MD.30.c016 | Set the Dictation keyboard shortcut | Customize | other | — | — | Choose this from the Shortcut pop-up menu, then press the desired keys; Option-Z is given as an example. | — |
| MD.30.c017 | Dictate in another language | Edit | button | — | — | Opens the language selection next to Languages. | Languages available for Dictation vary by language and region. |
| MD.30.c018 | Dictate in another language | Languages | list | Languages and regions available for Dictation | — | Select a language and region to add it; deselect a language to remove it. | Dictation is not available in all languages or regions, and features vary. |
| MD.30.c019 | Change the microphone used for Dictation | Microphone source | popup | Automatic; available microphone devices | — | Choose which microphone Dictation uses. Automatic uses the device the Mac is most likely to use for Dictation. | — |

Shortcuts stated: Press and release the Microphone key to start Dictation; press and hold it to activate Siri (Siri must be enabled).; Edit > Start Dictation; Press Escape, the Microphone key (if available), or the Dictation keyboard shortcut to stop Dictation.; Press the Globe key to switch Dictation languages when multiple languages are set up.; Option-Z is an example of a custom Dictation shortcut.

Notes: The page does not state a macOS version. The page notes that Dictation availability and features vary by language and region. On a Mac with Apple silicon, you can use the keyboard while dictating; dictation resumes after you stop typing. Dictation stops automatically when no speech is detected for 30 seconds. Voice Control uses speech for text entry when enabled; standard macOS Dictation isn’t available at the same time. The page’s sentence about the “Press fn key to” setting is truncated, so its full label and behavior are not provided.

### Keyboard: Keyboard settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/kbdm162/27/mac/27) · topic `kbdm162` · path: Apple menu > System Settings > Keyboard · "Open … for me" link: no

Use Keyboard settings to customize keyboard behavior and shortcuts, add input sources, and configure Dictation.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.30.c020 | Keyboard | Key repeat rate | slider | — | — | Set how quickly characters repeat when you press and hold certain keyboard keys. | — |
| MD.30.c021 | Keyboard | Delay until repeat | slider | — | — | Set how long to wait before characters repeat when you press and hold certain keyboard keys. | — |
| MD.30.c022 | Keyboard | Adjust keyboard brightness in low light | checkbox | — | — | Automatically adjust the level of keyboard backlighting in low-light conditions. | If your Mac has a backlit keyboard. |
| MD.30.c023 | Keyboard | Keyboard brightness | slider | — | — | Adjust the level of backlighting. | If your Mac has a backlit keyboard. |
| MD.30.c024 | Keyboard | Turn keyboard backlight off after inactivity | popup | Choose how long your Mac is idle before the keyboard backlighting is automatically turned off. | — | Set the idle time before the keyboard backlighting turns off automatically. | If your Mac has a backlit keyboard. |
| MD.30.c025 | Keyboard | Press fn key to | popup | Change Input Source; Show Emoji & Symbols; Start Dictation; Do Nothing | — | Choose what happens when you press the key on the keyboard. Change Input Source switches between keyboard layouts for writing in other languages. Show Emoji & Symbols opens the Character Viewer. Start Dictation starts dictation when you press the key twice. Do Nothing means nothing happens. | You may be asked to enable Dictation first when choosing Start Dictation. |
| MD.30.c026 | Keyboard | Keyboard navigation | checkbox | — | — | Press keys to move the focus between controls on the screen. Press Tab to move forward and Shift-Tab to move backward. | — |
| MD.30.c027 | Keyboard | Touch Bar Settings | button | — | — | Customize the Touch Bar and Control Strip. | If your Mac has a Touch Bar. |
| MD.30.c028 | Keyboard | Keyboard Shortcuts | button | — | — | Browse keyboard shortcuts, turn them on or off, or customize them. | — |
| MD.30.c029 | Text Input | Edit | button | — | — | Add input sources for typing in other languages and change options for specific sources. | — |
| MD.30.c030 | Text Input | Text Replacements | button | — | — | View the list of words and phrases to replace as you type and what to replace them with. | — |
| MD.30.c031 | Dictation | Dictation | toggle | On; Off | — | Turn Dictation on or off. | — |
| MD.30.c032 | Dictation | Languages | list | Choose one or more languages. | — | Choose Dictation languages. A downloaded language can be removed. | Dictation is not available in all languages. |
| MD.30.c033 | Dictation | Microphone source | popup | Automatic; available microphones | — | Choose a microphone. Automatic uses the device your Mac is most likely to use for Dictation. | — |
| MD.30.c034 | Dictation | Shortcut | popup | — | — | Choose a shortcut to start Dictation. | — |
| MD.30.c035 | Dictation | Auto-punctuation | toggle | — | — | Automatically insert commas, periods, and question marks as you dictate. | Not available in all languages. |
| MD.30.c036 | Dictation | About Ask Siri, Dictation & Privacy | link | — | — | Learn when Dictation may send things you say and other information to Apple to process requests, and how you can control what data is stored. The linked information also describes sharing or stopping sharing audio recordings with Apple. | — |

Shortcuts stated: Tab; Shift-Tab; Press the fn key twice to start Dictation when that action is selected.

Notes: The supplied page text does not state a macOS version. The page text begins the instructions for opening Keyboard settings but omits the rest of the sentence; the navigation path shown here uses the standard System Settings path. The supplied text contains a merged label, “Press fn key toorPress”; “Press fn key to” is used as the readable label. Dictation language availability varies; the page refers to the macOS Feature Availability webpage. The page links to related guidance for keyboard repeat, backlighting, input sources, text replacement, Dictation, and shortcuts.

#### Independent macOS 27 reports for MD.30

| ID | Kind | Pane | Claim | Source |
|---|---|---|---|---|
| MD.30.x01 | identifier | Keyboard | The System Settings link ending in `com.apple.Keyboard-Settings.extension` opens Keyboard. | T4 [Unpixel](https://unpixel.tech/blog/url-schemes-mac/) |
| MD.30.x02 | identifier | Keyboard | The page says both `com.apple.Keyboard` and `com.apple.Keyboard-Settings` work to open keyboard settings. | T4 [MacMost](https://macmost.com/mac-settings-links) |
| MD.30.x03 | removed | Keyboard > Dictation | The Auto-punctuation switch is gone. The article says automatic punctuation is presumably always on and that Apple says it is greatly improved in macOS 27. | T4 [geeksmodo](https://geeksmodo.com/macos-system-settings/) |
| MD.30.x04 | behavior | Settings > Profiles > (profile) > Keyboard | The setting “Shift Return sends Meta Return” is enabled by default and causes Shift-Return to insert an ESC before Return is sent. | T4 [Michael Tsai](https://mjtsai.com/blog/2026/09/14/macos-27-golden-gate/) |
| MD.30.x05 | restriction | Keyboard | The Spotlight keyboard shortcut can be changed in System Settings > Keyboard > Keyboard Shortcuts > Spotlight; the article says there is no evident way to change the dedicated Siri shortcut. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.30.x06 | identifier | Headphone, Keyboard, Trackpad, Mouse, Game Controller, CDs & DVDs, and Printers & Scanners | The listed commands open these settings sections. | T2 [GitHub](https://github.com/bvanpeski/SystemPreferences/blob/main/macos_preferencepanes-Ventura.md) |
| MD.30.x07 | identifier | Keyboard | The Keyboard pane is in the System Settings sidebar. | T4 [AbilityNet](https://api.abilitynet.org.uk/how-to-prevent-accidentally-typing-the-same-characters-in-macos-27-golden-gate) |
| MD.30.x08 | behavior | Keyboard | The Key repeat rate slider controls key repeat speed; moving it all the way to the left turns Key Repeat off. | T4 [AbilityNet](https://api.abilitynet.org.uk/how-to-prevent-accidentally-typing-the-same-characters-in-macos-27-golden-gate) |
| MD.30.x09 | behavior | Keyboard | The Delay until repeat slider changes how long it takes before keys start repeating. | T4 [AbilityNet](https://api.abilitynet.org.uk/how-to-prevent-accidentally-typing-the-same-characters-in-macos-27-golden-gate) |
| MD.30.x10 | restriction | Keyboard | If the settings are managed by someone else, such as an IT department or administrator, the user may need to contact them to access the settings or get help. | T4 [AbilityNet](https://api.abilitynet.org.uk/how-to-prevent-accidentally-typing-the-same-characters-in-macos-27-golden-gate) |
| MD.30.x11 | behavior | Keyboard > Keyboard Shortcuts | Keyboard Shortcuts are configured in System Settings > Keyboard > Keyboard Shortcuts and can include custom combinations for commands, system functions, or apps. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.30.x12 | behavior | Keyboard > Dictation | Dictation settings are in System Settings > Keyboard > Dictation; the article says advanced dictation features and voice commands can be customized. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.30.x13 | moved | MDM Restrictions payload > Keyboard Settings (DDM) | The listed keyboard restriction settings are deprecated in the MDM Restrictions payload; the article gives Keyboard Settings as their DDM location. | T4 [Microsoft Tech Community](https://techcommunity.microsoft.com/blog/IntuneCustomerSuccess/microsoft-intune-and-apple-os-27-new-settings-support-and-platform-changes/4559764) |

## 13.31 MD.31 Mouse

Status: **PARTIAL**. Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13).


### Mouse: Mouse settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mh29222/27/mac/27) · topic `mh29222` · path: Apple menu > System Settings > Mouse · "Open … for me" link: no

Use Mouse settings to customize mouse actions and gestures, including tracking, clicking, zooming, and swiping. A wireless mouse must be connected with your Mac to display the settings listed.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.31.c001 | Point & Click | Tracking speed | other | — | — | Adjust the speed at which the pointer follows the movement of your mouse. | A wireless mouse must be connected with your Mac to display the settings listed. |
| MD.31.c002 | Point & Click | Natural scrolling | other | — | — | Move the contents of a window in the same direction as your fingers. | A wireless mouse must be connected with your Mac to display the settings listed. |
| MD.31.c003 | Point & Click | Secondary click | other | — | — | Choose which side of the mouse to click for a secondary click (also called Control-click on Mac or right-click on Windows). | A wireless mouse must be connected with your Mac to display the settings listed. |
| MD.31.c004 | Point & Click | Smart zoom | other | — | — | Double-tap with one finger to zoom in or out. | A wireless mouse must be connected with your Mac to display the settings listed. |
| MD.31.c005 | More Gestures | Swipe between pages | other | — | — | Choose the gesture to use to move between pages in a document. | A wireless mouse must be connected with your Mac to display the settings listed. |
| MD.31.c006 | More Gestures | Swipe between full-screen applications | other | — | — | Swipe with two fingers to move between apps in full screen. | A wireless mouse must be connected with your Mac to display the settings listed. |
| MD.31.c007 | More Gestures | Mission Control | other | — | — | Double-tap with two fingers to enter Mission Control. | A wireless mouse must be connected with your Mac to display the settings listed. |
| MD.31.c008 | Advanced | Pointer acceleration | toggle | — | — | When this option is turned on, the pointer moves more quickly for fast mouse movements and more precisely for slow mouse movements. | A wireless mouse must be connected with your Mac to display the settings listed. |

Notes: The page mentions the popular topics “Connect a wireless mouse” and “Set up secondary click (called right-click on Windows).”

## 13.32 MD.32 Trackpad

Status: **PARTIAL**. Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13).


### Trackpad: Change Trackpad settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlp1226/27/mac/27) · topic `mchlp1226` · path: Apple menu > System Settings > Trackpad · "Open … for me" link: yes

Use Trackpad settings to adjust pointer movement and customize trackpad gestures. You need a Mac with a built-in trackpad or a wireless trackpad connected to your Mac; available options depend on the Mac or trackpad.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.32.c001 | Point & Click | Tracking speed | slider | — | — | Set the tracking speed of the pointer when you move it across the screen. | Requires a built-in trackpad or a wireless trackpad connected to the Mac. Available options may vary by Mac or trackpad. |
| MD.32.c002 | Point & Click | Click | popup | — | — | Set how firmly you must press the trackpad to take action. | Requires a built-in trackpad or a wireless trackpad connected to the Mac. Available options may vary by Mac or trackpad. |
| MD.32.c003 | Point & Click | Quiet Click | checkbox | — | — | Turn on this option if you don’t want your trackpad to make a sound when you click. | Requires a built-in trackpad or a wireless trackpad connected to the Mac. Available options may vary by Mac or trackpad. |
| MD.32.c004 | Point & Click | Force Click and haptic feedback | checkbox | — | — | Turn on this option to enable force click and tactile feedback (such as when aligning objects in some apps). | Requires a built-in trackpad or a wireless trackpad connected to the Mac. Available options may vary by Mac or trackpad. |
| MD.32.c005 | Point & Click | Look up & data detectors | popup | — | — | Choose a gesture to use to quickly look up a word or to do quick tasks with certain types of data, such as dates and addresses. | Requires a built-in trackpad or a wireless trackpad connected to the Mac. Available options may vary by Mac or trackpad. |
| MD.32.c006 | Point & Click | Secondary click | popup | — | — | Choose a gesture to use for a secondary click (also called Control-click on Mac or right-click on Windows). | Requires a built-in trackpad or a wireless trackpad connected to the Mac. Available options may vary by Mac or trackpad. |
| MD.32.c007 | Point & Click | Tap to click | checkbox | — | — | Tap the trackpad with one finger to perform a click. | Requires a built-in trackpad or a wireless trackpad connected to the Mac. Available options may vary by Mac or trackpad. |
| MD.32.c008 | Scroll & Zoom | Natural scrolling | checkbox | — | — | Move the contents of a window in the same direction as your fingers. | Requires a built-in trackpad or a wireless trackpad connected to the Mac. Available options may vary by Mac or trackpad. |
| MD.32.c009 | Scroll & Zoom | Zoom in or out | checkbox | — | — | Pinch two fingers closed to zoom in, or spread two fingers to zoom out. | Requires a built-in trackpad or a wireless trackpad connected to the Mac. Available options may vary by Mac or trackpad. |
| MD.32.c010 | Scroll & Zoom | Smart zoom | checkbox | — | — | Double-tap with two fingers to zoom in or out. | Requires a built-in trackpad or a wireless trackpad connected to the Mac. Available options may vary by Mac or trackpad. |
| MD.32.c011 | Scroll & Zoom | Rotate | checkbox | — | — | Use two fingers to rotate items on the screen. | Requires a built-in trackpad or a wireless trackpad connected to the Mac. Available options may vary by Mac or trackpad. |
| MD.32.c012 | More Gestures | Swipe between pages | popup | — | — | Choose the gesture to use to move between pages in a document. | Requires a built-in trackpad or a wireless trackpad connected to the Mac. Available options may vary by Mac or trackpad. |
| MD.32.c013 | More Gestures | Swipe between full-screen applications | popup | — | — | Choose the gesture to use to move between apps in full screen. | Requires a built-in trackpad or a wireless trackpad connected to the Mac. Available options may vary by Mac or trackpad. |
| MD.32.c014 | More Gestures | Notification Center | checkbox | — | — | Swipe left from the right edge of the trackpad to show Notification Center. | Requires a built-in trackpad or a wireless trackpad connected to the Mac. Available options may vary by Mac or trackpad. |
| MD.32.c015 | More Gestures | Mission Control | popup | — | — | Choose the gesture to use to open Mission Control. | Requires a built-in trackpad or a wireless trackpad connected to the Mac. Available options may vary by Mac or trackpad. |
| MD.32.c016 | More Gestures | App Exposé | popup | — | — | Choose the gesture to use to open Exposé. | Requires a built-in trackpad or a wireless trackpad connected to the Mac. Available options may vary by Mac or trackpad. |
| MD.32.c017 | More Gestures | Show Desktop | checkbox | — | — | Spread your thumb and three fingers to reveal your computer’s desktop. | Requires a built-in trackpad or a wireless trackpad connected to the Mac. Available options may vary by Mac or trackpad. |

Notes: The page does not state a macOS version. Options shown may vary depending on the Mac or trackpad. See also: Use Multi-Touch gestures on your Mac; If the pointer doesn’t move when using the trackpad with Mac; Look up words on Mac; Turn on three finger drag for your Mac trackpad.

### Gestures: View and customize mouse or trackpad gestures on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mh35869/27/mac/27) · topic `mh35869` · path: Apple menu · "Open … for me" link: yes

With a Magic Mouse or Apple trackpad, you can view available gestures, see an animation of each gesture, and customize or turn off gestures in Mouse or Trackpad settings.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.32.c018 | View and customize gestures | Open Mouse settings for me | link | — | — | Opens Mouse settings. | A wireless mouse must be connected with your Mac. |
| MD.32.c019 | View and customize gestures | Open Trackpad settings for me | link | — | — | Opens Trackpad settings. | A wireless trackpad must be connected with your Mac, or you must be using a Mac with a built-in trackpad. |
| MD.32.c020 | View and customize gestures | gestures | other | — | — | You can browse available gestures, view an animation demonstrating each gesture, and customize or turn off gestures. | Use a Magic Mouse or Apple trackpad. |

Notes: The page does not state a macOS version. The page says to choose Apple menu but does not give a fuller navigation path. For descriptions of individual gestures, the page refers to Change Trackpad settings or Mouse settings.

## 13.33 MD.33 Game Controllers

Status: **PARTIAL**. Only Games User Guide coverage; MDM identifier com.apple.Game-Controller-Settings.extension listed by Apple (GAP-B4-10).


### Game Controllers (Games app guide): Connect a game controller to Mac

T1 [Games User Guide](https://support.apple.com/guide/games/connect-a-game-controller-devf8cec167c/mac) · topic `devf8cec167c` · path: System Settings > Bluetooth; System Settings > Game Controllers · "Open … for me" link: yes

Connect a compatible game controller to your Mac using Bluetooth or a wired connection. Customize supported controller buttons or pair two controllers with Buddy Controller.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.33.c001 | Pair a Bluetooth game controller | Connect | button | — | — | Connects the controller shown in the Bluetooth device list. | The controller must be in discovery mode. |
| MD.33.c002 | Pair a Bluetooth game controller | Accept | button | — | — | Accepts a pairing request if prompted. | — |
| MD.33.c003 | Pair a Bluetooth game controller | (unlabelled) | field | A series of numbers | — | Enter the requested numbers and press Enter if prompted. | Shown if requested during pairing. |
| MD.33.c004 | Customize the game controller buttons | Select the controller you want to customize. | list | Connected controllers | — | Selects a controller to customize. | The controller must be connected to the Mac; button customization is available if the controller supports it. |
| MD.33.c005 | Customize the game controller buttons | Select the actions and share gestures you want to use with your controller. | other | Actions and share gestures | — | Choose actions for designated controller buttons. Examples given include taking a screenshot or recording a brief video of gameplay. | The controller must support button customization and be connected to the Mac. |
| MD.33.c006 | Add a second game controller and get help from a friend | Buddy Controller | button | — | — | Opens Buddy Controller setup. | Connect two compatible game controllers to the Mac using Bluetooth or a wired connection. |
| MD.33.c007 | Add a second game controller and get help from a friend | Choose the primary controller, then choose the secondary controller. | list | Connected controllers; choose one as the primary controller and another as the secondary controller. | — | Pairs two controllers to control the same player character. | Buddy Controller is compatible with any game that supports game controllers. |

Notes: The page does not state a macOS version. Depending on the Mac model and controller type, a USB adapter (sold separately) may be needed for a wired controller. No defaults or keyboard shortcuts are stated.

## 13.34 MD.34 Printers & Scanners

Status: **PARTIAL**. Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13).


### Printers & Scanners: Printers & Scanners settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/prtct004/27/mac/27) · topic `prtct004` · path: Apple menu > System Settings > Printers & Scanners · "Open … for me" link: yes

Set up printers and scanners, check ink levels, monitor print jobs, and manage printer settings.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.34.c001 | Popular topics | Default printer | other | Printers set up for use with the Mac. | The most recently used printer at that network location. | Specifies the printer to use as the default printer. | — |
| MD.34.c002 | Popular topics | Default paper size | other | Paper sizes; none are listed. | — | Choose the paper size loaded in the printer. | — |
| MD.34.c003 | Popular topics | Printers | list | Printers the Mac is set up to use. | — | Select a printer to view or change its name, location, printer model, or driver version; open its Printer Queue or Options and Supplies information; share it on a network; or allow others to use it. Control-click a printer to see printer info, set it as the default printer, remove it from the list, or reset the printing system. | For others to use a shared printer, they must use a Mac, Windows, or UNIX computer and be on the same local network. |
| MD.34.c004 | Popular topics | Add Printer, Scanner, or Fax button | button | Choose a device from the list, or enter its information in the dialog. Most printers and scanners are listed under Default. You can also add an IP printer or Windows printer. | — | Adds a printer, scanner, fax, or multifunction printer to the device list. | — |

## 13.35 MD.35 Menu Bar (if separate in 27)

Status: **PARTIAL**. Menu Bar confirmed as its own sidebar pane in 27 (resolves GAP-006).


### Menu Bar > customize: Customize the menu bar on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchl4af84660/27/mac/27) · topic `mchl4af84660` · path: Apple menu > System Settings > Menu Bar · "Open … for me" link: yes

Add menu bar icons from System Settings or Control Center, or remove icons you added from either source.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.35.c001 | Add an icon to the menu bar | Open Menu Bar settings for me | link | — | — | Opens Menu Bar settings to add icons for settings and controls. | — |
| MD.35.c002 | Add an icon to the menu bar | Siri, Weather, and Time Machine | other | — | — | Examples of settings and controls whose icons can be added from System Settings. | — |
| MD.35.c003 | Add an icon to the menu bar | Edit Controls | button | — | — | Opens Control Center editing so items can be dragged to the menu bar. | — |
| MD.35.c004 | Add an icon to the menu bar | Controls Gallery or Control Center | list | — | — | Drag an item from either source to the menu bar. Examples include Quick Note, Put Display to Sleep, and Color filters. | — |
| MD.35.c005 | Add an icon to the menu bar | Quick Note | other | — | — | Example of a Control Center item that can be added to the menu bar. | — |
| MD.35.c006 | Add an icon to the menu bar | Put Display to Sleep | other | — | — | Example of a Control Center item that can be added to the menu bar. | — |
| MD.35.c007 | Add an icon to the menu bar | Color filters | other | — | — | Example of a Control Center item that can be added to the menu bar. | — |
| MD.35.c008 | Remove an icon from the menu bar | Open Menu Bar settings for me | link | — | — | Opens Menu Bar settings to remove an icon added from System Settings. | — |
| MD.35.c009 | Remove an icon from the menu bar | Edit Controls | button | — | — | Opens Control Center editing; drag an item off the menu bar to remove it. | — |

### Menu Bar: Change Menu Bar settings on Mac

T1 [Mac User Guide](https://support.apple.com/guide/mac-help/mchlad96d366/27/mac/27) · topic `mchlad96d366` · path: Apple menu > System Settings > Menu Bar · "Open … for me" link: no

Choose which items appear in the menu bar, when the menu bar is hidden, and how recent items are displayed. More items are available in Control Center; items can also be dragged to the menu bar while customizing Control Center.

| ID | Section | Label | Type | Options | Default | Description | Conditions / effects |
|---|---|---|---|---|---|---|---|
| MD.35.c010 | General options | Automatically hide and show the menu bar | popup | Always; On Desktop Only; In Full Screen Only; Never | — | Choose when the menu bar is hidden and shown. Move the pointer to the top of the screen to show it when hidden. | — |
| MD.35.c011 | General options | Show menu bar background | checkbox | Show your wallpaper behind the menu bar; show the menu bar background | — | Choose whether to show your wallpaper behind the menu bar or show the menu bar background. | — |
| MD.35.c012 | General options | Recent documents, applications, and servers | popup | — | — | Choose how many items appear in the Recent Items menu command and in app menus such as Open Recent in TextEdit’s File menu. | — |
| MD.35.c013 | Menu Bar Controls | Clock | info | — | — | The clock is always shown in the menu bar. | — |
| MD.35.c014 | Menu Bar Controls | Clock Options | button | — | — | Open options for how the date and time are displayed in the menu bar clock. | — |
| MD.35.c015 | Menu Bar Controls | Siri | checkbox | — | — | Select to show in the menu bar. | — |
| MD.35.c016 | Menu Bar Controls | Spotlight | checkbox | — | — | Select to show in the menu bar. | — |
| MD.35.c017 | Menu Bar Controls | Wi-Fi | checkbox | — | — | Select to show in the menu bar. | — |
| MD.35.c018 | Menu Bar Controls | Bluetooth® | checkbox | — | — | Select to show in the menu bar. | — |
| MD.35.c019 | Menu Bar Controls | Battery | checkbox | — | — | Select to show Battery status in the menu bar. | Available if you’re using a Mac laptop. |
| MD.35.c020 | Menu Bar Controls | Battery Options | button | — | — | Open options for showing the percentage of battery charge and when to show the energy mode. | The Battery setting is described for Mac laptops. |
| MD.35.c021 | Menu Bar Controls | Battery percentage | info | — | — | The percentage is always shown when Battery is in Control Center. | — |
| MD.35.c022 | Menu Bar Controls | Focus | checkbox | — | — | Select to show in the menu bar. | — |
| MD.35.c023 | Menu Bar Controls | Focus | popup | Show When Active; Always Show | — | Choose when to show the status icon in the menu bar. | — |
| MD.35.c024 | Menu Bar Controls | Screen Mirroring | checkbox | — | — | Select to show in the menu bar. | — |
| MD.35.c025 | Menu Bar Controls | Screen Mirroring | popup | Show When Active; Always Show | — | Choose when to show the status icon in the menu bar. | — |
| MD.35.c026 | Menu Bar Controls | Display | checkbox | — | — | Select to show in the menu bar. | — |
| MD.35.c027 | Menu Bar Controls | Display | popup | Show When Active; Always Show | — | Choose when to show the status icon in the menu bar. | — |
| MD.35.c028 | Menu Bar Controls | Sound | checkbox | — | — | Select to show in the menu bar. | — |
| MD.35.c029 | Menu Bar Controls | Sound | popup | Show When Active; Always Show | — | Choose when to show the status icon in the menu bar. | — |
| MD.35.c030 | Menu Bar Controls | Now Playing | checkbox | — | — | Select to show in the menu bar. | — |
| MD.35.c031 | Menu Bar Controls | Now Playing | popup | Show When Active; Always Show | — | Choose when to show the status icon in the menu bar. | — |
| MD.35.c032 | Menu Bar Controls | Fast User Switching | checkbox | — | — | Select to show in the menu bar. | — |
| MD.35.c033 | Menu Bar Controls | Fast User Switching | popup | Full Name; Account Name; Icon | — | Choose whether the menu bar shows the user’s full name, account name, or the Fast User Switching icon. | — |
| MD.35.c034 | Menu Bar Controls | Time Machine | checkbox | — | — | Select to show in the menu bar. | — |
| MD.35.c035 | Menu Bar Controls | VPN | checkbox | — | — | Select to show in the menu bar. | — |
| MD.35.c036 | Menu Bar Controls | Weather | checkbox | — | — | Select to show in the menu bar. | — |
| MD.35.c037 | Allow in the Menu Bar | [Application name] | checkbox | — | — | Select to show an application’s menu bar item. | The items listed depend on applications that provide menu bar items. |
| MD.35.c038 | Reset Option | Reset Control Center | button | — | — | Return the Control Center to its original settings. | Resets Control Center settings. |

Notes: The supplied page text does not state a macOS version. The navigation instruction in the supplied text is truncated after “choose Apple menu”; the path shown includes the Menu Bar pane. More menu bar items are available in Control Center. Menu bar items can also be dragged directly to the menu bar when customizing Control Center.

#### Independent macOS 27 reports for MD.35

| ID | Kind | Pane | Claim | Source |
|---|---|---|---|---|
| MD.35.x01 | new | Menu Bar | A new switch controls whether macOS displays suggestions when Add Controls is clicked. Turning it off removes the suggestions. | T4 [geeksmodo](https://geeksmodo.com/macos-system-settings/) |
| MD.35.x02 | behavior | System Settings › Menu Bar | The “Show menu bar background” switch puts a translucent, Sequoia-style backdrop behind the menu titles. | T4 [publicspace.net](https://www.publicspace.net/blog/macos-27-menu-bar/) |
| MD.35.x03 | behavior | System Settings › Menu Bar | The “Show menu bar background” switch remains available in macOS 27. | T4 [publicspace.net](https://www.publicspace.net/blog/macos-27-menu-bar/) |
| MD.35.x04 | changed | Menu Bar | The menu bar has its own pane under System Settings > Menu Bar. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |

Resolution of Part 1 GAP-006: Menu Bar is its own sidebar pane in macOS 27, at `Apple menu > System Settings > Menu Bar` ([Menu Bar settings](https://support.apple.com/guide/mac-help/mchlad96d366/27/mac/27)), with a customisation topic ([Customize the menu bar](https://support.apple.com/guide/mac-help/mchl4af84660/27/mac/27)). This is corroborated by [Timing](https://timingapp.com/blog/system-settings-mac/) and [publicspace.net](https://www.publicspace.net/blog/macos-27-menu-bar/) (T4).

## 13.37 MD.36 App Settings conventions (Command-Comma)

Status: **COMPLETE** (macOS evidence). HIG design detail continues in Part 7 (B6).

| ID | Rule | Source |
|---|---|---|
| MD.36.r01 | Command-Comma (,) opens "settings (preferences) for the front app". | T1 [Mac keyboard shortcuts](https://support.apple.com/en-us/102650) (published 2026-09-14) |
| MD.36.r02 | To change app options, open the app, click the app's name in the menu bar, then choose Settings. "Some apps don't offer settings." | T1 [Customize your Mac with System Settings](https://support.apple.com/guide/mac-help/mh15217/27/mac/27) |
| MD.36.r03 | "Include a settings item in the App menu." For document-level options, use the File menu. Avoid settings buttons in a window's toolbar. | T1 [Apple HIG](https://developer.apple.com/design/human-interface-guidelines/settings) |
| MD.36.r04 | Use a noncustomizable toolbar of panes that stays visible and always shows the active button. | T1 [Apple HIG](https://developer.apple.com/design/human-interface-guidelines/settings) |
| MD.36.r05 | Dim the settings window's minimize and maximize buttons. | T1 [Apple HIG](https://developer.apple.com/design/human-interface-guidelines/settings) |
| MD.36.r06 | The window title reflects the visible pane. A single-pane window is titled "App Name Settings". | T1 [Apple HIG](https://developer.apple.com/design/human-interface-guidelines/settings) |
| MD.36.r07 | "Restore the most recently viewed pane." | T1 [Apple HIG](https://developer.apple.com/design/human-interface-guidelines/settings) |
| MD.36.r08 | "Add only the most rarely changed options to the system-provided Settings app." Consider a button that opens it directly. | T1 [Apple HIG](https://developer.apple.com/design/human-interface-guidelines/settings) |
| MD.36.r09 | Shortcuts that open system panes: Option–Brightness Up opens Displays, Option–Mission Control opens Mission Control settings, Option–Volume Up opens Sound, and Option–Keyboard Brightness Up opens Keyboard. | T1 [Mac keyboard shortcuts](https://support.apple.com/en-us/102650) |

Boundary (Rule 5): MD.36.r01–r02 describe OS behaviour. MD.36.r03–r08 are developer guidance and do not guarantee behaviour in third-party apps.

## 13.38 MD.37 `x-apple.systempreferences` URL schemes

Status: **PARTIAL**. `macos_settings_urls.csv` has 209 rows covering 121 pane identifiers and 88 pane+anchor rows.

Key facts:

1. Apple documents no URL-scheme reference for System Settings. Apple does list Settings-extension identifiers (for example `com.apple.Siri-Settings.extension` and `com.apple.Game-Controller-Settings.extension`), but only as values for the MDM key `DisabledSystemSettings` ([SystemPreferences payload](https://developer.apple.com/documentation/devicemanagement/systempreferences)). 145 CSV rows match an identifier on that Apple list.
2. Community sources say the URL form is `x-apple.systempreferences:<id>?<anchor>`. They say the sidebar's identifiers live in `/System/Applications/System Settings.app/Contents/Resources/Sidebar.plist`, and that deeper sub-pane identifiers can be found by running `strings` on the app binary ([jaywcjlove README](https://github.com/jaywcjlove/SystemSettings-URLs-macOS/blob/main/README.md), T2). Unpixel lists macOS 26 extension IDs such as `com.apple.Keyboard-Settings.extension` ([Unpixel](https://unpixel.tech/blog/url-schemes-mac/), T4).
3. Chromium's macOS utilities open System Settings with extension IDs on macOS 13 and later, and use legacy `com.apple.preference.*` IDs on macOS 12 and earlier. Examples include `com.apple.Accessibility-Settings.extension` with anchor `Captioning`, and `com.apple.Network-Settings.extension` with anchor `Proxies` ([Chromium mac_util.mm](https://chromium.googlesource.com/chromium/src/+/HEAD/base/mac/mac_util.mm), T2).
4. On macOS 27, the legacy link `com.apple.preference.security?Privacy_Accessibility` reportedly still opens the renamed privacy pane ([sorla issue #74](https://github.com/markstrom/sorla/issues/74), T2 issue tracker). This is the only row tested on 27. Every other row is "UNVERIFIED on 27".

| Identifier kind | Rows |
|---|---|

| Legacy preference-pane ID (com.apple.preference*) | 92 |
| Other/unclassified ID | 61 |
| Settings extension ID (*.extension, macOS 13+ style) | 56 |

Sources merged: [Chromium source](https://chromium.googlesource.com/chromium/src/+/HEAD/base/mac/mac_util.mm), [Apple Developer Documentation](https://developer.apple.com/documentation/devicemanagement/systempreferences), [GitHub](https://github.com/bvanpeski/SystemPreferences/blob/main/macos_preferencepanes-Ventura.md), [GitHub](https://github.com/jaywcjlove/SystemSettings-URLs-macOS/blob/main/README.md), [GitHub](https://github.com/markstrom/sorla/issues/74), [GitHub](https://github.com/rapjul/macOS-System-Prefs), [MacMost](https://macmost.com/mac-settings-links), [Unpixel](https://unpixel.tech/blog/url-schemes-mac/), [macOS Adventures](https://www.macosadventures.com/2022/12/05/how-to-open-every-section-of-macos-ventura-system-settings/).

## 13.39 MD.38 Configuration profiles / MDM

Status: **PARTIAL**.

| ID | Finding | Source |
|---|---|---|
| MD.38.f01 | The System Preferences payload (`com.apple.systempreferences`) provides `DisabledPreferencePanes` and `EnabledPreferencePanes` (legacy pane IDs) and `DisabledSystemSettings` (extension IDs). Apple marks all three deprecated as of macOS 13. | T1 [SystemPreferences payload](https://developer.apple.com/documentation/devicemanagement/systempreferences) |
| MD.38.f02 | If `DisabledSystemSettings` is present, the device ignores the two legacy keys. One extension can supply several sections, and disabling it disables all of them. | T1 [SystemPreferences payload](https://developer.apple.com/documentation/devicemanagement/systempreferences) |
| MD.38.f03 | The Restrictions payload (`com.apple.applicationaccess`) gates specific settings. Examples: `allowLocalUserCreation` (no user creation in System Settings, macOS 14+), `allowStartupDiskModification`, `allowTimeMachineBackup`, `allowEraseContentAndSettings` (disables Erase All Content and Settings), `forceAutomaticDateAndTime` (locks Set Automatically on) and `allowWallpaperModification`. | T1 [Restrictions](https://developer.apple.com/documentation/devicemanagement/restrictions) |
| MD.38.f04 | Software-update deferral keys such as `enforcedSoftwareUpdateMajorOSDeferredInstallDelay` and `forceDelayedMajorSoftwareUpdates` are listed as removed in macOS 27. | T1 [Restrictions](https://developer.apple.com/documentation/devicemanagement/restrictions) |
| MD.38.f05 | Mac restrictions are on by default unless marked "Default is off". Many Apple Intelligence restrictions (macOS 15.1–15.4) require supervision. | T1 [Restrictions for Mac](https://support.apple.com/guide/deployment/restrictions-for-mac-depba790e53/web) |
| MD.38.f06 | Apple is ending several legacy MDM payloads with OS 27, and DDM replaces them in Intune's settings catalog. New 27 DDM areas include app/binary launch control, Intelligence, Accessibility, Safari, Siri and login window. | T4 [Microsoft Tech Community](https://techcommunity.microsoft.com/blog/IntuneCustomerSuccess/microsoft-intune-and-apple-os-27-new-settings-support-and-platform-changes/4559764) |
| MD.38.f07 | Legacy software-update management and deferrals stop working in 27.0, and older PPPC Accessibility grants are removed. Users can switch off Accessibility permissions that MDM granted. | T4 [Dr Logic](https://drlogic.com/article/macos-27-is-live-the-security-changes-that-will-quietly-break-on-your-mac-fleet/) |
| MD.38.f08 | MDM enrollment "does not appear to work completely" in Golden Gate (author report). | T4 [Michael Tsai](https://mjtsai.com/blog/2026/09/14/macos-27-golden-gate/) |
| MD.38.f09 | The user-visible surface is General > Device Management, which lists installed profiles. | T1 [Device Management settings](https://support.apple.com/guide/mac-help/mh35474/27/mac/27) |

| MD.38.f10 | In macOS 27, the pane formerly called Accessibility is named “Device control and data access.” | T4 [Addigy](https://addigy.com/resources/managing-os-27-in-the-field/) |
| MD.38.f11 | The permission can no longer be allowlisted via an MDM profile in macOS 27. | T4 [Addigy](https://addigy.com/resources/managing-os-27-in-the-field/) |
| MD.38.f12 | The app settings declaration prompts the user when the app first launches; the user must choose whether to allow the permission. | T4 [Addigy](https://addigy.com/resources/managing-os-27-in-the-field/) |
| MD.38.f13 | After the user chooses Allow for an app in the tested user-channel setup, the prompt does not appear again for that app. | T4 [Addigy](https://addigy.com/resources/managing-os-27-in-the-field/) |
| MD.38.f14 | Clicking the notification about the permission takes the user to the Device control and data access settings. | T4 [Addigy](https://addigy.com/resources/managing-os-27-in-the-field/) |
| MD.38.f15 | Third-party remote-control tools require the user to approve keyboard and mouse control; Live Desktop is not affected. | T4 [Addigy](https://addigy.com/resources/managing-os-27-in-the-field/) |
| MD.38.f16 | Bestimmte MDM-Kommandos stehen unter macOS 27 und iOS 27 nicht mehr wie gewohnt zur Verfügung. | T4 [Born City](https://borncity.com/news/macos-27-und-ios-27-apple-streicht-vertraute-mdm-kommandos/) |
| MD.38.f17 | Organisationen wird empfohlen, eingesetzte Konfigurationsprofile auf veraltete Instruktionen zu prüfen. | T4 [Born City](https://borncity.com/news/macos-27-und-ios-27-apple-streicht-vertraute-mdm-kommandos/) |
| MD.38.f18 | Nach einer Neuinstallation können bei Kalender, Karten, Wo ist?, Home und Wetter Aufforderungen zur Aktivierung von Benachrichtigungen ausbleiben; als Abhilfe soll die Aktivierung manuell in den Systemeinstellungen erfolgen. | T4 [Born City](https://borncity.com/news/macos-27-und-ios-27-apple-streicht-vertraute-mdm-kommandos/) |

## 13.40 Independent macOS 27 change reports (all claims)

Every claim from the non-Apple pages read in full is listed here. They are T4/T5 claims, not verified facts.

| ID | Kind | Pane | Claim | Source |
|---|---|---|---|---|
| MD.X001 | changed | Battery | The single Low Power Mode menu was replaced by separate menus for battery and power-adapter use. Each offers Automatic, Low Power, or High Power. | T4 [geeksmodo](https://geeksmodo.com/macos-system-settings/) |
| MD.X002 | new | General > AutoFill & Passwords | A new switch controls which apps can supply AutoFill data. Passwords is the only option by default and is turned on; third-party password managers may also appear. | T4 [geeksmodo](https://geeksmodo.com/macos-system-settings/) |
| MD.X003 | new | Accessibility > Display | A new switch adds borders around toolbar controls, Search fields, and other interface items to make their boundaries clearer. | T4 [geeksmodo](https://geeksmodo.com/macos-system-settings/) |
| MD.X004 | changed | Accessibility > Display | “Show borders” replaces the previous setting for toolbar button shapes. | T4 [geeksmodo](https://geeksmodo.com/macos-system-settings/) |
| MD.X005 | renamed | Accessibility > Subtitles and Captioning | The Captions category is replaced by Subtitles and Captioning. | T4 [geeksmodo](https://geeksmodo.com/macos-system-settings/) |
| MD.X006 | new | Accessibility > Subtitles and Captioning | Three controls were added to adjust when and where subtitles or captions appear. | T4 [geeksmodo](https://geeksmodo.com/macos-system-settings/) |
| MD.X007 | new | Menu Bar | A new switch controls whether macOS displays suggestions when Add Controls is clicked. Turning it off removes the suggestions. | T4 [geeksmodo](https://geeksmodo.com/macos-system-settings/) |
| MD.X008 | new | Wallpaper | macOS 27 includes new static and dynamic wallpapers and a new screen saver. | T4 [geeksmodo](https://geeksmodo.com/macos-system-settings/) |
| MD.X009 | new | Lock Screen | A new switch is on by default, preserving the behavior of Tahoe and earlier. Turning it off hides usernames and photos when the Mac is locked. | T4 [geeksmodo](https://geeksmodo.com/macos-system-settings/) |
| MD.X010 | removed | Keyboard > Dictation | The Auto-punctuation switch is gone. The article says automatic punctuation is presumably always on and that Apple says it is greatly improved in macOS 27. | T4 [geeksmodo](https://geeksmodo.com/macos-system-settings/) |
| MD.X011 | new | Appearance | System Settings adds a slider to adjust Liquid Glass transparency, from ultra-clear to fully tinted. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MD.X012 | new | Siri | A new Voice section in System Settings lets users adjust Siri's expressiveness and speaking pace with sliders. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MD.X013 | new | Accessibility | A new Show Borders option makes interface element boundaries easier to see; the article does not specify its location in System Settings. | T4 [MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/) |
| MD.X014 | new | Appearance | A new Liquid Glass slider lets users adjust interface transparency, from ultra-clear to more opaque and tinted, or choose an intermediate setting. | T4 [iGeeksBlog](https://www.igeeksblog.com/macos-27-features/) |
| MD.X015 | behavior | General > Software Update | macOS 27 Golden Gate is available through System Settings > General > Software Update. | T4 [iGeeksBlog](https://www.igeeksblog.com/macos-27-features/) |
| MD.X016 | renamed | System Settings > Privacy & Security | The Accessibility pane was renamed to Device Control and Data Access. | T4 [Michael Tsai](https://mjtsai.com/blog/2026/09/14/macos-27-golden-gate/) |
| MD.X017 | behavior | Settings > Profiles > (profile) > Keyboard | The setting “Shift Return sends Meta Return” is enabled by default and causes Shift-Return to insert an ESC before Return is sent. | T4 [Michael Tsai](https://mjtsai.com/blog/2026/09/14/macos-27-golden-gate/) |
| MD.X018 | behavior | System Settings | The app can be resized in any direction, made narrower than its default size, and its sidebar can be adjusted to provide more space. This capability is hidden behind a feature flag. | T4 [Michael Tsai](https://mjtsai.com/blog/2026/09/14/macos-27-golden-gate/) |
| MD.X019 | removed | Apple Intelligence | A toggle that previously let users turn off Apple Intelligence and delete its downloaded models is no longer available. | T4 [Michael Tsai](https://mjtsai.com/blog/2026/09/14/macos-27-golden-gate/) |
| MD.X020 | bug | Device management | MDM enrollment does not appear to work completely in macOS Golden Gate. | T4 [Michael Tsai](https://mjtsai.com/blog/2026/09/14/macos-27-golden-gate/) |
| MD.X021 | behavior | System Settings → General → About | The Intel-based Apps details view lists Intel-based apps on the Mac. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/13/) |
| MD.X022 | new | Screen Time | Screen Time adds Time Allowances for app categories. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/13/) |
| MD.X023 | new | Screen Time | Screen Time adds Schedules for app access during certain times of the day. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/13/) |
| MD.X024 | new | Screen Time | Ask to Browse lets children request permission to visit websites. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/13/) |
| MD.X025 | changed | Screen Time | Screen Time expands protections against gore and violence alongside existing nudity protections. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/13/) |
| MD.X026 | restriction | Screen Time | The new Screen Time features require all devices in a Family Sharing group to be on Apple’s latest series of operating systems. | T4 [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/13/) |
| MD.X027 | new | Appearance | macOS 27 adds a Liquid Glass appearance slider, expanding beyond the two styles available previously. | T4 [9to5Mac](https://9to5mac.com/2026/09/22/macos-27-gives-you-more-control-over-liquid-glass/) |
| MD.X028 | behavior | Appearance | The slider defaults to a medium setting. | T4 [9to5Mac](https://9to5mac.com/2026/09/22/macos-27-gives-you-more-control-over-liquid-glass/) |
| MD.X029 | new | Appearance | The system invites users to adjust the slider during initial setup. | T4 [9to5Mac](https://9to5mac.com/2026/09/22/macos-27-gives-you-more-control-over-liquid-glass/) |
| MD.X030 | behavior | Appearance | Users can adjust the slider later in System Settings. | T4 [9to5Mac](https://9to5mac.com/2026/09/22/macos-27-gives-you-more-control-over-liquid-glass/) |
| MD.X031 | behavior | Appearance | The slider provides a live preview of each setting. | T4 [9to5Mac](https://9to5mac.com/2026/09/22/macos-27-gives-you-more-control-over-liquid-glass/) |
| MD.X032 | behavior | Appearance | Three notches indicate the clearest, medium, and most opaque settings, and the slider can also be set between notches. | T4 [9to5Mac](https://9to5mac.com/2026/09/22/macos-27-gives-you-more-control-over-liquid-glass/) |
| MD.X033 | behavior | Appearance | The slider changes the appearance of the Dock, including how clearly the desktop wallpaper shows through. | T4 [9to5Mac](https://9to5mac.com/2026/09/22/macos-27-gives-you-more-control-over-liquid-glass/) |
| MD.X034 | behavior | Appearance | The slider changes how dim desktop widgets look. | T4 [9to5Mac](https://9to5mac.com/2026/09/22/macos-27-gives-you-more-control-over-liquid-glass/) |
| MD.X035 | behavior | Appearance | The slider changes the appearance of system Now Playing bars in apps such as Apple Music and Podcasts. | T4 [9to5Mac](https://9to5mac.com/2026/09/22/macos-27-gives-you-more-control-over-liquid-glass/) |
| MD.X036 | behavior | Appearance | The slider also affects sidebars, toolbars, and toolbar buttons in apps such as Photos and Safari. | T4 [9to5Mac](https://9to5mac.com/2026/09/22/macos-27-gives-you-more-control-over-liquid-glass/) |
| MD.X037 | behavior | Appearance | One slider controls Liquid Glass appearances across the system, including the Dock, widgets, and window chrome. | T4 [9to5Mac](https://9to5mac.com/2026/09/22/macos-27-gives-you-more-control-over-liquid-glass/) |
| MD.X038 | behavior | System Settings > Appearance | The Appearance pane has a “Liquid Glass” section with a slider for adjusting transparency. | T4 [Apple World Today](https://appleworld.today/2026/09/how-to-adjust-the-transparency-of-liquid-glass-in-macos-27/) |
| MD.X039 | behavior | System Settings > Appearance > Liquid Glass | Dragging the slider left gives a clearer, more transparent look; dragging it right increases frosting, tint, and opacity. | T4 [Apple World Today](https://appleworld.today/2026/09/how-to-adjust-the-transparency-of-liquid-glass-in-macos-27/) |
| MD.X040 | behavior | System Settings > Accessibility > Display | Turning on “Reduce Transparency” makes transparency almost completely off, producing a solid background. | T4 [Apple World Today](https://appleworld.today/2026/09/how-to-adjust-the-transparency-of-liquid-glass-in-macos-27/) |
| MD.X041 | new | System Settings | A slider for adjusting the intensity of the Liquid Glass effect is available during setup and in System Settings. Its default is the middle position. | T4 [Ars Technica](https://arstechnica.com/gadgets/2026/09/macos-27-golden-gate-the-ars-technica-review/) |
| MD.X042 | behavior | System Settings | The Liquid Glass slider ranges from a glassier appearance on the left to reduced sheen and translucency on the right. | T4 [Ars Technica](https://arstechnica.com/gadgets/2026/09/macos-27-golden-gate-the-ars-technica-review/) |
| MD.X043 | changed | System Settings | The previous binary Liquid Glass setting was labeled “Clear/Tinted.” | T4 [Ars Technica](https://arstechnica.com/gadgets/2026/09/macos-27-golden-gate-the-ars-technica-review/) |
| MD.X044 | new | System Settings > Siri | On Macs with an M3 or newer processor and at least 12GB of RAM, Siri settings include pace and expressivity sliders for its voice. | T4 [Ars Technica](https://arstechnica.com/gadgets/2026/09/macos-27-golden-gate-the-ars-technica-review/) |
| MD.X045 | behavior | System Settings > Siri | Pace controls how quickly Siri speaks, and expressivity controls how much inflection its voice uses. | T4 [Ars Technica](https://arstechnica.com/gadgets/2026/09/macos-27-golden-gate-the-ars-technica-review/) |
| MD.X046 | restriction | System Settings > Siri | Pace and expressivity options are currently available only for American and British accents; non-American accents have fewer voice options. | T4 [Ars Technica](https://arstechnica.com/gadgets/2026/09/macos-27-golden-gate-the-ars-technica-review/) |
| MD.X047 | behavior | System Settings | Siri conversation retention defaults to “Forever” and can be changed to one year or 30 days. | T4 [Ars Technica](https://arstechnica.com/gadgets/2026/09/macos-27-golden-gate-the-ars-technica-review/) |
| MD.X048 | behavior | System Settings | Settings let users change the number of visible lines in conversation previews and choose whether the Siri app opens to past conversations or a new conversation. | T4 [Ars Technica](https://arstechnica.com/gadgets/2026/09/macos-27-golden-gate-the-ars-technica-review/) |
| MD.X049 | new | System Settings > General > About | A new “Intel-based Apps” section lists recently launched Intel-exclusive apps; universal binaries do not appear in the list. | T4 [Ars Technica](https://arstechnica.com/gadgets/2026/09/macos-27-golden-gate-the-ars-technica-review/) |
| MD.X050 | removed | System Settings | The Apple Intelligence toggle for turning the feature off and removing its downloaded models is no longer available in macOS 27. | T4 [Ars Technica](https://arstechnica.com/gadgets/2026/09/macos-27-golden-gate-the-ars-technica-review/) |
| MD.X051 | behavior | System Settings > Accessibility | In macOS 26 Tahoe, “reduce transparency” could force the hard-style title-bar divider across apps, while also switching off other transparency effects. | T4 [Ars Technica](https://arstechnica.com/gadgets/2026/09/macos-27-golden-gate-the-ars-technica-review/) |
| MD.X052 | behavior | System Settings > Storage | After a fresh install, the Storage tab reported about 14GB for Apple Intelligence on the M1 MacBook Air and MacBook Neo, and 22.42GB on the M3 MacBook Air. | T4 [Ars Technica](https://arstechnica.com/gadgets/2026/09/macos-27-golden-gate-the-ars-technica-review/) |
| MD.X053 | behavior | System Settings | ChatGPT remains available as an extension, with an option to use a voice different from Siri’s. | T4 [Ars Technica](https://arstechnica.com/gadgets/2026/09/macos-27-golden-gate-the-ars-technica-review/) |
| MD.X054 | behavior | Siri AI | Users can opt out of the Siri AI beta; the article says they are then offered the choice to turn Siri off or use “Siri Classic.” | T4 [Ars Technica](https://arstechnica.com/gadgets/2026/09/macos-27-golden-gate-the-ars-technica-review/) |
| MD.X055 | behavior | Siri | The pane appears with a new icon and the new version is available only when Language is set to English (United States); other languages, including English (Canada), show the old icon and Tahoe-era Siri features. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.X056 | removed | Siri | The new pane no longer has an Apple Intelligence switch or a Siri switch at the top. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.X057 | behavior | Siri | To disable Siri, scroll to the bottom, click the red Turn Off Siri button, read its description of what will stop working, and click Turn Off Siri again. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.X058 | behavior | Siri | When Siri is turned off, Visual Intelligence continues to work with more limited capabilities; Writing Tools, Image Playground, and Shortcuts continue to work. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.X059 | new | Siri | The Siri App setting Opens To selects Recent Conversations or A New Conversation. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.X060 | changed | Siri | Keep Conversations offers 30 Days, 1 Year, or Forever; the pane no longer has a Delete Siri & Dictation History button, and history items can instead be deleted in the Siri app. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.X061 | new | Siri | Preview offers None or 1–5 lines. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.X062 | renamed | Siri | Requests replaces the previous Listen for pop-up menu, whose choices included Siri, Hey Siri, or Off. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.X063 | changed | Siri | Responses offers Spoken Response or Silent Response; macOS Tahoe also offered Automatic, Prefer Silent Responses, and Prefer Spoken Responses. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.X064 | renamed | Siri | App Access replaces Siri Suggestions & Privacy for specifying each app’s Siri settings. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.X065 | new | Siri | The pane includes additional ChatGPT options for users who connect Siri to ChatGPT. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.X066 | moved | Siri | The Automatic Visual Look Up switch is in the Siri pane. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.X067 | removed | Siri | macOS 27 no longer lets users set the keyboard shortcut used to invoke Siri; macOS Tahoe did. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.X068 | behavior | Siri | Siri can be invoked with ⌘-Space, the Spotlight shortcut, or a second dedicated shortcut. Holding the dedicated shortcut speaks to Siri even when Requests is turned off. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.X069 | restriction | Keyboard | The Spotlight keyboard shortcut can be changed in System Settings > Keyboard > Keyboard Shortcuts > Spotlight; the article says there is no evident way to change the dedicated Siri shortcut. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.X070 | behavior | iCloud | Siri conversation syncing is controlled in System Settings > Your Name > iCloud > See All > Siri. Sync this Mac is enabled by default and can be disabled. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.X071 | behavior | iCloud | Keep Conversations is synced with the main Siri retention setting; changing it in iCloud changes it globally, not only for iCloud. | T4 [geeksmodo](https://geeksmodo.com/macos-siri-settings/) |
| MD.X072 | changed | System Settings › Appearance | The Liquid Glass setting is a slider from fully transparent to opaque for the whole interface. | T4 [publicspace.net](https://www.publicspace.net/blog/macos-27-menu-bar/) |
| MD.X073 | behavior | System Settings › Menu Bar | The “Show menu bar background” switch puts a translucent, Sequoia-style backdrop behind the menu titles. | T4 [publicspace.net](https://www.publicspace.net/blog/macos-27-menu-bar/) |
| MD.X074 | behavior | System Settings › Appearance | Moving the Liquid Glass slider toward opaque makes the glass effect frosted throughout the interface, including the menu bar when its background is shown. | T4 [publicspace.net](https://www.publicspace.net/blog/macos-27-menu-bar/) |
| MD.X075 | behavior | System Settings › Accessibility › Display | Reduce transparency replaces blur with a flat fill across the interface: light grey in light mode and near-black in dark mode. | T4 [publicspace.net](https://www.publicspace.net/blog/macos-27-menu-bar/) |
| MD.X076 | behavior | System Settings › Control Center | There is a switch for each of Apple’s listed menu bar icons, including Wi-Fi, Bluetooth, Battery, Sound, and Focus. | T4 [publicspace.net](https://www.publicspace.net/blog/macos-27-menu-bar/) |
| MD.X077 | behavior | System Settings › Displays | A scaled resolution that does not use the notch rows blacks out that strip, at the cost of those rows of screen. | T4 [publicspace.net](https://www.publicspace.net/blog/macos-27-menu-bar/) |
| MD.X078 | behavior | System Settings › Menu Bar | The “Show menu bar background” switch remains available in macOS 27. | T4 [publicspace.net](https://www.publicspace.net/blog/macos-27-menu-bar/) |
| MD.X079 | renamed | Privacy & Security | On macOS 27, the permission pane Sorla needs is called “Device Control and Data Access” in English and “Enhetskontroll och dataåtkomst” in Swedish, instead of “Accessibility” / “Hjälpmedel.” The issue says this is confirmed by the SecurityPrivacyExtension localized strings for key ACCESSIBILITY on macOS 27.0 (26A428). | T2 [GitHub](https://github.com/markstrom/sorla/issues/74) |
| MD.X080 | renamed | Privacy & Security | The requested naming by running macOS version is “Accessibility” / “Hjälpmedel” on macOS 14–26 and “Device Control and Data Access” / “Enhetskontroll och dataåtkomst” on macOS 27+. | T2 [GitHub](https://github.com/markstrom/sorla/issues/74) |
| MD.X081 | behavior | Privacy & Security | The deep link x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility opens the pane directly and still works on macOS 27. | T2 [GitHub](https://github.com/markstrom/sorla/issues/74) |
| MD.X082 | removed | System Settings > Apple Intelligence & Siri | The earlier single Apple Intelligence control at the top of this pane is no longer documented in macOS 27; users must change feature controls individually. | T5 [WindowsForum](https://windowsforum.com/news/macos-27-removes-apple-intelligence-master-toggle-wont-free-storage.445490/) |
| MD.X083 | behavior | System Settings > Siri | To turn off Siri AI (Beta), select “Turn Off Siri” and confirm by selecting it again. | T5 [WindowsForum](https://windowsforum.com/news/macos-27-removes-apple-intelligence-master-toggle-wont-free-storage.445490/) |
| MD.X084 | behavior | System Settings > Siri | After turning Siri off, users can select “Turn On Siri” and then choose “Use Siri Classic” to switch to the older assistant. | T5 [WindowsForum](https://windowsforum.com/news/macos-27-removes-apple-intelligence-master-toggle-wont-free-storage.445490/) |
| MD.X085 | behavior | System Settings > Siri | Call Context and suggestions from apps are controlled through Siri’s app-access settings rather than through a single global control. | T5 [WindowsForum](https://windowsforum.com/news/macos-27-removes-apple-intelligence-master-toggle-wont-free-storage.445490/) |
| MD.X086 | behavior | System Settings > Siri > App Access | A user-written walkthrough says app access can be limited per app by turning off “Learn from this application” and, when shown, “Show Siri Suggestions in application.” The article says the walkthrough is not Apple’s own wording. | T5 [WindowsForum](https://windowsforum.com/news/macos-27-removes-apple-intelligence-master-toggle-wont-free-storage.445490/) |
| MD.X087 | behavior | System Settings > Notifications | Notification summaries can be disabled by turning off “Summarize Notifications.” | T5 [WindowsForum](https://windowsforum.com/news/macos-27-removes-apple-intelligence-master-toggle-wont-free-storage.445490/) |
| MD.X088 | behavior | System Settings > Screen Time > Content & Privacy > Intelligence & Siri | The “Apple Intelligence” option controls access to features including Image Creation, Writing Tools, and Intelligence Extensions. | T5 [WindowsForum](https://windowsforum.com/news/macos-27-removes-apple-intelligence-master-toggle-wont-free-storage.445490/) |
| MD.X089 | behavior | System Settings > Screen Time > Content & Privacy > Intelligence & Siri | Separate Siri options control Siri & Dictation, explicit language, and web search content in Siri. | T5 [WindowsForum](https://windowsforum.com/news/macos-27-removes-apple-intelligence-master-toggle-wont-free-storage.445490/) |
| MD.X090 | behavior | System Settings > Screen Time | For a family member’s device, a parent or guardian in Family Sharing first selects the family member from the pop-up menu, then opens Content & Privacy and Intelligence & Siri. | T5 [WindowsForum](https://windowsforum.com/news/macos-27-removes-apple-intelligence-master-toggle-wont-free-storage.445490/) |
| MD.X091 | behavior | System Settings > Screen Time | Screen Time restrictions can be protected with a passcode, which keeps a restricted feature blocked if someone turns its app-level toggle back on. | T5 [WindowsForum](https://windowsforum.com/news/macos-27-removes-apple-intelligence-master-toggle-wont-free-storage.445490/) |
| MD.X092 | changed | System Settings > Screen Time > Intelligence & Siri | The article notes that Screen Time labels on macOS 27 may differ slightly from Tahoe-era labels and advises checking the pane on the Mac. | T5 [WindowsForum](https://windowsforum.com/news/macos-27-removes-apple-intelligence-master-toggle-wont-free-storage.445490/) |
| MD.X093 | behavior | System Settings > Privacy & Security | An Apple Community author suggests keeping the Apple Intelligence report setting turned on to check whether Apple Intelligence is functioning. | T5 [WindowsForum](https://windowsforum.com/news/macos-27-removes-apple-intelligence-master-toggle-wont-free-storage.445490/) |
| MD.X094 | behavior | Storage | To check storage, open System Settings > General > Storage. The page shows space used by Applications, Documents, and Photos, but does not identify Apple Intelligence storage separately. | T4 [PCMag UK](https://uk.pcmag.com/macos/167598/is-apple-intelligence-stealing-your-macs-storage-heres-how-to-turn-it-off) |
| MD.X095 | behavior | Apple Intelligence & Siri | On macOS 26 or earlier, Apple Intelligence can be disabled in System Settings > Apple Intelligence & Siri > Apple Intelligence by toggling it off. | T4 [PCMag UK](https://uk.pcmag.com/macos/167598/is-apple-intelligence-stealing-your-macs-storage-heres-how-to-turn-it-off) |
| MD.X096 | behavior | Screen Time | On macOS 27, the article says to enable Screen Time, open Content & Privacy, turn its toggle off, then open Siri and select Turn Off Siri. It says Siri can then be changed to Siri Classic. | T4 [PCMag UK](https://uk.pcmag.com/macos/167598/is-apple-intelligence-stealing-your-macs-storage-heres-how-to-turn-it-off) |
| MD.X097 | behavior | Siri | On macOS 27, System Settings > Siri offers the choices Don't Allow Siri or Siri Classic, depending on preference. | T4 [PCMag UK](https://uk.pcmag.com/macos/167598/is-apple-intelligence-stealing-your-macs-storage-heres-how-to-turn-it-off) |
| MD.X098 | behavior | Siri | On macOS 27, individual items in the Capabilities list can be turned off. | T4 [PCMag UK](https://uk.pcmag.com/macos/167598/is-apple-intelligence-stealing-your-macs-storage-heres-how-to-turn-it-off) |
| MD.X099 | behavior | Notifications | Summarize Notifications can be turned off at System Settings > Notifications > Summarize Notifications. | T4 [PCMag UK](https://uk.pcmag.com/macos/167598/is-apple-intelligence-stealing-your-macs-storage-heres-how-to-turn-it-off) |
| MD.X100 | behavior | Messages | Summarize Messages can be turned off in Messages > Settings. | T4 [PCMag UK](https://uk.pcmag.com/macos/167598/is-apple-intelligence-stealing-your-macs-storage-heres-how-to-turn-it-off) |
| MD.X101 | behavior | Mail | Summarize Message Previews can be turned off in Mail > Settings > Viewing. | T4 [PCMag UK](https://uk.pcmag.com/macos/167598/is-apple-intelligence-stealing-your-macs-storage-heres-how-to-turn-it-off) |
| MD.X102 | behavior | Mail | Personalize Smart Replies can be turned off in Mail > Settings > Composing. | T4 [PCMag UK](https://uk.pcmag.com/macos/167598/is-apple-intelligence-stealing-your-macs-storage-heres-how-to-turn-it-off) |
| MD.X103 | behavior | Apple Intelligence & Siri | On macOS 26 or earlier, turning off Apple Intelligence should free disk space; a reboot may be needed before the available storage reflects the change. | T4 [PCMag UK](https://uk.pcmag.com/macos/167598/is-apple-intelligence-stealing-your-macs-storage-heres-how-to-turn-it-off) |
| MD.X104 | behavior | Siri | On macOS 27, disabling Apple Intelligence features and switching to classic Siri will not automatically clear storage. The article says System Data might decrease over time, but this is not guaranteed. | T4 [PCMag UK](https://uk.pcmag.com/macos/167598/is-apple-intelligence-stealing-your-macs-storage-heres-how-to-turn-it-off) |
| MD.X105 | removed | Device management; specific pane not stated | Legacy software update management, update deferrals, and older Background Security Improvements restrictions no longer work in macOS 27.0. Updates need to use declarative device management. | T4 [Dr Logic](https://drlogic.com/article/macos-27-is-live-the-security-changes-that-will-quietly-break-on-your-mac-fleet/) |
| MD.X106 | removed | Privacy permissions; specific pane not stated | Older PPPC profile keys are being phased out, and Accessibility permissions granted through the old profile format are removed. | T4 [Dr Logic](https://drlogic.com/article/macos-27-is-live-the-security-changes-that-will-quietly-break-on-your-mac-fleet/) |
| MD.X107 | behavior | Privacy permissions; specific pane not stated | Users can switch off Accessibility permissions that device management granted automatically. | T4 [Dr Logic](https://drlogic.com/article/macos-27-is-live-the-security-changes-that-will-quietly-break-on-your-mac-fleet/) |
| MD.X108 | new | Device management; specific pane not stated | macOS 27 adds built-in management of which apps and command-line tools are allowed to run on a Mac. | T4 [Dr Logic](https://drlogic.com/article/macos-27-is-live-the-security-changes-that-will-quietly-break-on-your-mac-fleet/) |
| MD.X109 | changed | Platform Single Sign-On; specific pane not stated | Platform Single Sign-On now reaches the login window and FileVault unlock. Touch ID or an Apple Watch can be required. | T4 [Dr Logic](https://drlogic.com/article/macos-27-is-live-the-security-changes-that-will-quietly-break-on-your-mac-fleet/) |
| MD.X110 | new | Device setup; specific pane not stated | Admins can confirm a Mac is fully configured before a new starter receives it. | T4 [Dr Logic](https://drlogic.com/article/macos-27-is-live-the-security-changes-that-will-quietly-break-on-your-mac-fleet/) |
| MD.X111 | new | Privacy & Security > Local Network | In macOS 27.2, users can add and remove apps from the Local Network list. | T4 [The Eclectic Light Company](https://eclecticlight.co/2026/10/22/local-network-privacy-is-getting-better/) |
| MD.X112 | behavior | Privacy & Security > Local Network | Users can control whether listed apps are allowed or denied local network access. | T4 [The Eclectic Light Company](https://eclecticlight.co/2026/10/22/local-network-privacy-is-getting-better/) |
| MD.X113 | behavior | Displays | On macOS 26 Tahoe, an HDR10-capable third-party display has a High Dynamic Range toggle in Settings → Displays. | T4 [9to5Mac](https://9to5mac.com/2026/10/05/macos-27-just-fixed-the-most-annoying-problem-for-third-party-hdr-displays/) |
| MD.X114 | behavior | Displays | On macOS 26 Tahoe, selecting a scaled resolution such as 2560 × 1440 disables HDR and switches the display back to Standard Dynamic Range. | T4 [9to5Mac](https://9to5mac.com/2026/10/05/macos-27-just-fixed-the-most-annoying-problem-for-third-party-hdr-displays/) |
| MD.X115 | changed | Displays | macOS 27 Golden Gate allows HDR to remain enabled while a scaled resolution such as 2560 × 1440 is selected; the article reports this worked on an M4 MacBook Pro. | T4 [9to5Mac](https://9to5mac.com/2026/10/05/macos-27-just-fixed-the-most-annoying-problem-for-third-party-hdr-displays/) |
| MD.X116 | restriction | Displays | The author could not select both HDR and a scaled resolution on an M1 Max MacBook Pro running macOS 27, suggesting hardware requirements. | T4 [9to5Mac](https://9to5mac.com/2026/10/05/macos-27-just-fixed-the-most-annoying-problem-for-third-party-hdr-displays/) |
| MD.X117 | renamed | Device control and data access | In macOS 27, the pane formerly called Accessibility is named “Device control and data access.” | T4 [Addigy](https://addigy.com/resources/managing-os-27-in-the-field/) |
| MD.X118 | restriction | Device control and data access | The permission can no longer be allowlisted via an MDM profile in macOS 27. | T4 [Addigy](https://addigy.com/resources/managing-os-27-in-the-field/) |
| MD.X119 | behavior | Device control and data access | The app settings declaration prompts the user when the app first launches; the user must choose whether to allow the permission. | T4 [Addigy](https://addigy.com/resources/managing-os-27-in-the-field/) |
| MD.X120 | behavior | Device control and data access | After the user chooses Allow for an app in the tested user-channel setup, the prompt does not appear again for that app. | T4 [Addigy](https://addigy.com/resources/managing-os-27-in-the-field/) |
| MD.X121 | behavior | Device control and data access | Clicking the notification about the permission takes the user to the Device control and data access settings. | T4 [Addigy](https://addigy.com/resources/managing-os-27-in-the-field/) |
| MD.X122 | behavior | Device control and data access | Third-party remote-control tools require the user to approve keyboard and mouse control; Live Desktop is not affected. | T4 [Addigy](https://addigy.com/resources/managing-os-27-in-the-field/) |
| MD.X123 | identifier | Keyboard | The Keyboard pane is in the System Settings sidebar. | T4 [AbilityNet](https://api.abilitynet.org.uk/how-to-prevent-accidentally-typing-the-same-characters-in-macos-27-golden-gate) |
| MD.X124 | behavior | Keyboard | The Key repeat rate slider controls key repeat speed; moving it all the way to the left turns Key Repeat off. | T4 [AbilityNet](https://api.abilitynet.org.uk/how-to-prevent-accidentally-typing-the-same-characters-in-macos-27-golden-gate) |
| MD.X125 | behavior | Keyboard | The Delay until repeat slider changes how long it takes before keys start repeating. | T4 [AbilityNet](https://api.abilitynet.org.uk/how-to-prevent-accidentally-typing-the-same-characters-in-macos-27-golden-gate) |
| MD.X126 | restriction | Keyboard | If the settings are managed by someone else, such as an IT department or administrator, the user may need to contact them to access the settings or get help. | T4 [AbilityNet](https://api.abilitynet.org.uk/how-to-prevent-accidentally-typing-the-same-characters-in-macos-27-golden-gate) |
| MD.X127 | removed | MDM | Bestimmte MDM-Kommandos stehen unter macOS 27 und iOS 27 nicht mehr wie gewohnt zur Verfügung. | T4 [Born City](https://borncity.com/news/macos-27-und-ios-27-apple-streicht-vertraute-mdm-kommandos/) |
| MD.X128 | behavior | Konfigurationsprofile | Organisationen wird empfohlen, eingesetzte Konfigurationsprofile auf veraltete Instruktionen zu prüfen. | T4 [Born City](https://borncity.com/news/macos-27-und-ios-27-apple-streicht-vertraute-mdm-kommandos/) |
| MD.X129 | bug | Systemeinstellungen | Nach einer Neuinstallation können bei Kalender, Karten, Wo ist?, Home und Wetter Aufforderungen zur Aktivierung von Benachrichtigungen ausbleiben; als Abhilfe soll die Aktivierung manuell in den Systemeinstellungen erfolgen. | T4 [Born City](https://borncity.com/news/macos-27-und-ios-27-apple-streicht-vertraute-mdm-kommandos/) |
| MD.X130 | behavior | System Settings → Privacy & Security → Analytics & Improvements | Turning off Improve Siri & Dictation stops Apple storing audio and transcripts of Siri and Dictation requests for quality review. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.X131 | behavior | System Settings → Privacy & Security → Analytics & Improvements | On Mac, turn off Improve Siri & Dictation; if the Improve Siri & Apple Intelligence toggle is present, turn it off too. The text says iOS 27 adds the latter toggle for model training. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.X132 | behavior | System Settings → Siri → App Access | Learn from this App controls how Siri learns from app use to make suggestions; turning it off for an app stops Siri suggesting actions or content based on use of that app. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.X133 | behavior | System Settings → Siri | The Siri app saves conversations and syncs them across devices through iCloud. Retention choices are forever, one year, or 30 days. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.X134 | behavior | System Settings → Screen Time → Content & Privacy → Siri | Siri Classic switches back to the previous Siri. Content & Privacy must be turned on first if it is off. Screen Time can also restrict individual AI features. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.X135 | restriction | Siri AI on-screen awareness | There is no setting to turn off Siri AI's on-screen awareness. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.X136 | behavior | System Settings → Privacy & Security → Analytics & Improvements | Turning off Share Mac Analytics stops daily diagnostics and usage data being sent to Apple, including hardware configuration, OS performance, and app usage patterns. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.X137 | behavior | System Settings → Privacy & Security → Analytics & Improvements | Turning off Share with App Developers stops third-party app crash data and usage statistics being sent to developers through Apple. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.X138 | renamed | System Settings → Privacy & Security | The pane formerly called Accessibility is now called Device Control and Data Access. Apps listed there can control the Mac and read what is on screen. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.X139 | behavior | System Settings → Privacy & Security → Device Control and Data Access | Review the listed apps and remove ones you do not recognize; this access is described as among the broadest an app can have. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.X140 | behavior | System Settings → Privacy & Security | Golden Gate denies apps' requests to read another developer's app data folders by default, without showing a permission prompt. Access can be granted in Privacy & Security. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.X141 | behavior | System Settings → Privacy & Security | Review app permissions for Camera, Microphone, Files and Folders, Full Disk Access, and Screen & System Audio Recording; access can be revoked. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.X142 | behavior | System Settings → Privacy & Security → Location Services | Location Services can remain on while per-app access is reviewed. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.X143 | behavior | System Settings → Privacy & Security → Location Services → System Services → Details | Turning off Significant Locations stops the Mac logging frequently visited places; turning off Mac Analytics keeps location data out of analytics reports sent to Apple. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.X144 | behavior | System Settings → Privacy & Security → Apple Advertising | Turning off Personalized Ads means ads in the App Store, Apple News, and Stocks are not based on interests, search behavior, or app downloads; it does not reduce the number of ads. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.X145 | behavior | System Settings → Privacy & Security → FileVault | FileVault encrypts the startup disk and is turned on automatically during setup with an Apple Account. Its recovery key is stored in the end-to-end encrypted Passwords app. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.X146 | behavior | System Settings → Network → Firewall | The Firewall is off by default. Turning it on blocks unsolicited incoming connections, but does not affect outgoing connections or normal browsing. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.X147 | behavior | System Settings → Privacy & Security → Background Security Improvements | Automatically installing Background Security Improvements lets Apple deliver small security fixes for Safari, WebKit, and other system components between full macOS updates. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.X148 | behavior | Safari → Settings → Privacy | Prevent cross-site tracking is on by default and blocks third-party cookies and cross-site tracking using Intelligent Tracking Prevention. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.X149 | behavior | Safari → Settings → Privacy | Hide IP address from trackers can be set to Trackers Only, or to Trackers and Websites with iCloud+. The broader option uses Private Relay; without iCloud+, the feature relies on local processing. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.X150 | behavior | Safari → Settings → Advanced | Turning off Privacy Preserving Ad Measurement prevents ad attribution data from being shared. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.X151 | removed | System Settings → Apple Intelligence | The earlier single switch that turned Apple Intelligence off and deleted its downloaded models is gone in Golden Gate; the models remain on disk, though individual features can be restricted. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.X152 | changed | System Settings → Privacy & Security | The default behavior for access to other developers' app data changed: requests that previously triggered a permission prompt are now denied, and access can be allowed in Privacy & Security. | T4 [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate) |
| MD.X153 | behavior | Appearance | Light, Dark, and Auto appearance modes are available in System Settings > Appearance. Auto switches between Light and Dark depending on the time of day. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X154 | behavior | Displays > Night Shift | Night Shift can be scheduled to shift display colors toward the warmer end of the spectrum after dark. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X155 | behavior | Appearance | Accent color is configured in System Settings > Appearance > Theme > Color and appears in buttons, selection highlights, and sidebar icons. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X156 | new | Appearance | macOS 27 adds a Liquid Glass slider under System Settings > Appearance > Liquid Glass to adjust the glass appearance from clearer to more tinted. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X157 | behavior | Wallpaper | Wallpaper is configured in System Settings > Wallpaper; the article also gives the screen saver path as System Settings > Wallpaper > Screen Saver… | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X158 | behavior | Trackpad | Trackpad settings are in System Settings > Trackpad and include gesture and secondary-click customization. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X159 | behavior | Desktop & Dock | Dock settings include size, position, minimizing when not in use, and app-opening animation options. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X160 | changed | Menu Bar | The menu bar has its own pane under System Settings > Menu Bar. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X161 | behavior | Desktop & Dock > Hot Corners | Hot Corners can be assigned actions such as Mission Control, Desktop view, or Screen Saver; moving the cursor to a corner activates the assigned function. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X162 | behavior | Focus | Focus modes can be created from presets or with Custom, then configured with a name, icon, and color. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X163 | behavior | Focus | Focus settings can specify which people and apps may send notifications and whether Time-Sensitive Notifications are allowed. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X164 | behavior | Focus | Focus Filters can change app behavior, including showing or hiding Mail accounts, Calendar sets, Safari tab groups, or Messages accounts. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X165 | behavior | Focus | Focus options include dimming the Lock Screen, hiding notification badges, or showing only certain widgets while Focus is active. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X166 | behavior | Focus | Focus can be scheduled by time, location, or app. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X167 | behavior | Focus | Share across devices can sync Focus settings with an iPhone, iPad, and Apple Watch; Focus status can tell contacts that notifications are silenced. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X168 | behavior | Focus | On supported Macs, Reduce Interruptions with Apple Intelligence can intelligently filter notifications. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X169 | behavior | Focus | The article says an app-based Focus trigger enables Focus when the selected app launches and disables it when the app closes or the user switches away. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X170 | behavior | Notifications | Notifications settings are in System Settings > Notifications and allow per-app alert customization, including banners, alerts, silent notifications, and Notification Center inclusion. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X171 | behavior | Sound | Sound settings include system alert sounds, input and output settings, interface feedback sounds, and an Alert volume slider. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X172 | behavior | Screen Time | Screen Time can show app and website usage, set Time Allowances, schedule app availability, restrict content, and generate usage reports. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X173 | behavior | Desktop & Dock | Mission Control settings are located in System Settings > Desktop & Dock. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X174 | behavior | Keyboard > Keyboard Shortcuts | Keyboard Shortcuts are configured in System Settings > Keyboard > Keyboard Shortcuts and can include custom combinations for commands, system functions, or apps. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X175 | behavior | Battery | The article describes battery settings for sleep, Power Nap, and balancing performance with battery life under System Settings > Battery. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X176 | behavior | Battery | The article says Optimized Battery Charging is enabled by clicking the information button next to Charging and toggling the option on. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X177 | new | Battery | On macOS 27, the charging sheet also has a Charge Limit slider from 80 to 100%; the article says setting it to 80% can help reduce battery aging. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X178 | behavior | Accessibility | Accessibility settings include vision, hearing, motor, and speech options, with examples such as voice commands, color filters, and screen zoom. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X179 | behavior | General > Login Items | Login Items are used to choose apps that launch automatically when the Mac starts. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X180 | behavior | Keyboard > Dictation | Dictation settings are in System Settings > Keyboard > Dictation; the article says advanced dictation features and voice commands can be customized. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X181 | behavior | Finder | Finder settings, accessed in Finder rather than System Settings, can control display of file paths, hard drives, and tags; hard drives can be shown on the desktop and in the Finder sidebar. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X182 | behavior | Privacy & Security | The article recommends reviewing app permissions in Privacy & Security. | T4 [Timing](https://timingapp.com/blog/system-settings-mac/) |
| MD.X183 | new | Settings catalog > App settings (DDM) | New DDM settings control which apps can launch on supervised iPhone, iPad, Apple TV, and Apple Vision Pro devices, and which binaries can run on supervised Macs. They also define organization-suggested privacy permission defaults. | T4 [Microsoft Tech Community](https://techcommunity.microsoft.com/blog/IntuneCustomerSuccess/microsoft-intune-and-apple-os-27-new-settings-support-and-platform-changes/4559764) |
| MD.X184 | new | Settings catalog > Intelligence settings (DDM) | New DDM settings control Apple Intelligence capabilities, app-specific features, and requirements for on-device dictation and translation. | T4 [Microsoft Tech Community](https://techcommunity.microsoft.com/blog/IntuneCustomerSuccess/microsoft-intune-and-apple-os-27-new-settings-support-and-platform-changes/4559764) |
| MD.X185 | new | Settings catalog > Accessibility settings (DDM) | New DDM settings configure organization-defined accessibility preferences, including display, text, motion, audio, and interaction options. | T4 [Microsoft Tech Community](https://techcommunity.microsoft.com/blog/IntuneCustomerSuccess/microsoft-intune-and-apple-os-27-new-settings-support-and-platform-changes/4559764) |
| MD.X186 | new | Settings catalog > Safari settings (DDM) | New DDM settings manage Safari behavior and privacy, including cookies, fraud warnings, history clearing, JavaScript, pop-ups, private browsing, summaries, new-tab start pages, and website camera or microphone permission defaults. | T4 [Microsoft Tech Community](https://techcommunity.microsoft.com/blog/IntuneCustomerSuccess/microsoft-intune-and-apple-os-27-new-settings-support-and-platform-changes/4559764) |
| MD.X187 | new | Settings catalog > Siri settings (DDM) | New DDM settings control Siri availability and behavior, including Siri AI features, user-generated content, access while locked, profanity filtering, and reduction of sensitive content. | T4 [Microsoft Tech Community](https://techcommunity.microsoft.com/blog/IntuneCustomerSuccess/microsoft-intune-and-apple-os-27-new-settings-support-and-platform-changes/4559764) |
| MD.X188 | new | Settings catalog > Content caching (DDM) | New DDM settings configure macOS Content Caching to store Apple-distributed software and iCloud content locally and configure clients, networks, parent and peer behavior, storage, and network use. | T4 [Microsoft Tech Community](https://techcommunity.microsoft.com/blog/IntuneCustomerSuccess/microsoft-intune-and-apple-os-27-new-settings-support-and-platform-changes/4559764) |
| MD.X189 | new | Settings catalog > DNS Proxy (DDM) | New DDM settings configure a DNS proxy network extension, including its provider app and vendor-defined settings, to route DNS requests through an organization-approved service. | T4 [Microsoft Tech Community](https://techcommunity.microsoft.com/blog/IntuneCustomerSuccess/microsoft-intune-and-apple-os-27-new-settings-support-and-platform-changes/4559764) |
| MD.X190 | new | Settings catalog > Web content filter (DDM) | New DDM settings configure a plug-in-based web content filter, including provider, authentication, and filtering settings. | T4 [Microsoft Tech Community](https://techcommunity.microsoft.com/blog/IntuneCustomerSuccess/microsoft-intune-and-apple-os-27-new-settings-support-and-platform-changes/4559764) |
| MD.X191 | new | Settings catalog > Login Window | A new MDM configuration customizes the macOS login window and sign-in experience, including information shown to users and available login options and controls. | T4 [Microsoft Tech Community](https://techcommunity.microsoft.com/blog/IntuneCustomerSuccess/microsoft-intune-and-apple-os-27-new-settings-support-and-platform-changes/4559764) |
| MD.X192 | new | Enrollment policies > Setup Assistant panes | The new enrollment policies experience lets Intune add Skip Keys, including Liquid Glass and Accessibility Appearance. | T4 [Microsoft Tech Community](https://techcommunity.microsoft.com/blog/IntuneCustomerSuccess/microsoft-intune-and-apple-os-27-new-settings-support-and-platform-changes/4559764) |
| MD.X193 | new | Device actions > Enhanced logging | Intune supports Apple's Enhanced Logging device action on supported supervised devices running a compatible OS release. Administrators can start an AppleCare diagnostic-log collection session with an AppleCare-provided token and monitor device-reported status through DDM. | T4 [Microsoft Tech Community](https://techcommunity.microsoft.com/blog/IntuneCustomerSuccess/microsoft-intune-and-apple-os-27-new-settings-support-and-platform-changes/4559764) |
| MD.X194 | removed | MDM payloads | Apple is ending support for the listed legacy MDM payloads with OS 27; the article recommends managing those workloads using DDM through the settings catalog. | T4 [Microsoft Tech Community](https://techcommunity.microsoft.com/blog/IntuneCustomerSuccess/microsoft-intune-and-apple-os-27-new-settings-support-and-platform-changes/4559764) |
| MD.X195 | moved | MDM Restrictions payload > App Settings (DDM) | The listed app restriction settings are deprecated in the MDM Restrictions payload; the article gives App Settings as their DDM location. | T4 [Microsoft Tech Community](https://techcommunity.microsoft.com/blog/IntuneCustomerSuccess/microsoft-intune-and-apple-os-27-new-settings-support-and-platform-changes/4559764) |
| MD.X196 | moved | MDM Restrictions payload > External Intelligence Settings (DDM) | The listed external intelligence restriction settings are deprecated in the MDM Restrictions payload; the article gives External Intelligence Settings as their DDM location. | T4 [Microsoft Tech Community](https://techcommunity.microsoft.com/blog/IntuneCustomerSuccess/microsoft-intune-and-apple-os-27-new-settings-support-and-platform-changes/4559764) |
| MD.X197 | moved | MDM Restrictions payload > Intelligence Settings (DDM) | The listed Apple Intelligence restriction settings are deprecated in the MDM Restrictions payload; the article gives Intelligence Settings as their DDM location. | T4 [Microsoft Tech Community](https://techcommunity.microsoft.com/blog/IntuneCustomerSuccess/microsoft-intune-and-apple-os-27-new-settings-support-and-platform-changes/4559764) |
| MD.X198 | moved | MDM Restrictions payload > Keyboard Settings (DDM) | The listed keyboard restriction settings are deprecated in the MDM Restrictions payload; the article gives Keyboard Settings as their DDM location. | T4 [Microsoft Tech Community](https://techcommunity.microsoft.com/blog/IntuneCustomerSuccess/microsoft-intune-and-apple-os-27-new-settings-support-and-platform-changes/4559764) |
| MD.X199 | moved | MDM Restrictions payload > Siri Settings (DDM) | The listed Siri restriction settings are deprecated in the MDM Restrictions payload; the article gives Siri Settings as their DDM location. | T4 [Microsoft Tech Community](https://techcommunity.microsoft.com/blog/IntuneCustomerSuccess/microsoft-intune-and-apple-os-27-new-settings-support-and-platform-changes/4559764) |
| MD.X200 | removed | Intune admin center > Apple software update workloads | Apple has ended support for MDM device configuration templates and software update commands and settings. Starting with the October (2610) release, Intune will remove the listed legacy update policies and reports from the admin center; the article recommends DDM instead. | T4 [Microsoft Tech Community](https://techcommunity.microsoft.com/blog/IntuneCustomerSuccess/microsoft-intune-and-apple-os-27-new-settings-support-and-platform-changes/4559764) |
| MD.X201 | behavior | Intune App SDK for iOS > App protection | With Intune App SDK for iOS v21.8.0 or later, the Screen capture app protection policy setting controls whether Siri onscreen awareness can access work or school data. Block prevents the Ask Siri option from appearing for organizational data; Allow is the default and lets users share organizational data through Siri onscreen awareness. | T4 [Microsoft Tech Community](https://techcommunity.microsoft.com/blog/IntuneCustomerSuccess/microsoft-intune-and-apple-os-27-new-settings-support-and-platform-changes/4559764) |
| MD.X202 | new | Intune app protection experience | The refreshed iOS experience adds clearer prompts and account context, and a Remove Account option for removing a blocked managed account and its organizational data from the app. The article says this is a user experience update only, with no changes to app functionality or app protection policies. | T4 [Microsoft Tech Community](https://techcommunity.microsoft.com/blog/IntuneCustomerSuccess/microsoft-intune-and-apple-os-27-new-settings-support-and-platform-changes/4559764) |

## 13.41 Gaps and contradictions (GAP-B4-xx)

| ID | Subject | Apple evidence | Contradicting / extra evidence | Resolution path |
|---|---|---|---|---|
| GAP-B4-01 | Sidebar order | Not published; the guide says items vary by Mac ([overview](https://support.apple.com/guide/mac-help/mh15217/27/mac/27)) | Sidebar.plist location ([jaywcjlove](https://github.com/jaywcjlove/SystemSettings-URLs-macOS/blob/main/README.md)) | VP-13: read Sidebar.plist on 26A434 |
| GAP-B4-02 | Privacy item name | 27 guide lists "Accessibility" under Privacy ([Privacy & Security](https://support.apple.com/guide/mac-help/mchl211c911f/27/mac/27)) | Renamed "Device Control and Data Access" ([Michael Tsai](https://mjtsai.com/blog/2026/09/14/macos-27-golden-gate/), [sorla #74](https://github.com/markstrom/sorla/issues/74), [Everyday Privacy](https://everydayprivacy.khaldoun.dev/os/macos-golden-gate)) | Live check; independent sources agree, so the guide is likely lagging |
| GAP-B4-03 | Accessibility > Display | "Show toolbar button shapes" ([Display accessibility](https://support.apple.com/guide/mac-help/unac089/27/mac/27)) | "Show borders" replaces it ([geeksmodo](https://geeksmodo.com/macos-system-settings/)); MacRumors also reports "Show Borders" ([MacRumors](https://www.macrumors.com/2026/09/17/new-features-changes-macos-27/)) | Live check |
| GAP-B4-04 | Captions | "Captions" ([Captions](https://support.apple.com/guide/mac-help/mh43180/27/mac/27)) | Renamed "Subtitles and Captioning", with three new controls ([geeksmodo](https://geeksmodo.com/macos-system-settings/)) | Live check |
| GAP-B4-05 | Apple Intelligence master switch | No single switch on the 27 restriction page; a single allow-toggle remains in Screen Time ([Screen Time](https://support.apple.com/guide/mac-help/mchl1885610b/27/mac/27)) | Removed from Settings ([Ars Technica](https://arstechnica.com/gadgets/2026/09/macos-27-golden-gate-the-ars-technica-review/), [WindowsForum](https://windowsforum.com/news/macos-27-removes-apple-intelligence-master-toggle-wont-free-storage.445490/), [Michael Tsai](https://mjtsai.com/blog/2026/09/14/macos-27-golden-gate/)) | Consistent once the Screen Time toggle is treated as a restriction rather than an on/off switch |
| GAP-B4-06 | Siri pane options | Responses: Prefer Silent / Automatic / Prefer Spoken; Requests popup ([Siri settings](https://support.apple.com/guide/mac-help/mchl3fd7fc15/27/mac/27)) | Spoken/Silent only, Keep Conversations, Preview lines, no custom shortcut, and the new pane only in English (US) ([geeksmodo](https://geeksmodo.com/macos-siri-settings/)) | Language-dependent UI; test en-US and en-GB/th-TH (VP-14) |
| GAP-B4-07 | Guide content inherited from earlier releases | Intel-only labels on the About page ([About](https://support.apple.com/guide/mac-help/mchlea7173f3/27/mac/27)) | macOS 27 is Apple-silicon-only (Part 1 baseline) | Treat 27 guide labels as T1 but possibly stale |
| GAP-B4-08 | Lock Screen user display | No switch for user names or photos ([Lock Screen](https://support.apple.com/guide/mac-help/mh11784/27/mac/27)) | New switch, on by default ([geeksmodo](https://geeksmodo.com/macos-system-settings/)) | Live check |
| GAP-B4-09 | Screen Saver location | Guide path `System Settings > Screen Saver` ([Screen Saver](https://support.apple.com/guide/mac-help/mchlp1227/27/mac/27)) | Reached through "Wallpaper > Screen Saver…" ([Timing](https://timingapp.com/blog/system-settings-mac/)) | VP-13 |
| GAP-B4-10 | Game Controllers | Documented only in the Games User Guide | MDM extension ID listed ([SystemPreferences payload](https://developer.apple.com/documentation/devicemanagement/systempreferences)) | Live check |
| GAP-B4-11 | AppleCare & Warranty, AutoFill & Passwords | No 27 guide topic; support articles [102607](https://support.apple.com/en-us/102607) and [120758](https://support.apple.com/en-us/120758) (the latter shows Sequoia) | geeksmodo reports a new AutoFill provider switch | Search the 27 guide again after updates |
| GAP-B4-12 | Legacy topics | `sfri40599` (Passwords) and `mtusr005` (login options) redirect to the guide home | — | NOT FOUND (27) |
| GAP-B4-13 | URL identifiers | Extension IDs exist only as MDM values ([SystemPreferences payload](https://developer.apple.com/documentation/devicemanagement/systempreferences)) | Community lists span macOS 13–26 | VP-15: open every CSV row on 26A434 |
| GAP-B4-14 | MDM successor for hiding panes | `DisabledSystemSettings` is deprecated with no successor named on that page | Intune reports DDM for 27 areas | Part 12 (Research I) |
| GAP-B4-15 | Point releases beyond baseline | — | Local Network list add/remove "in macOS 27.2" ([The Eclectic Light Company](https://eclecticlight.co/2026/10/22/local-network-privacy-is-getting-better/)) | Out of 27.0.1 scope; tracked |
| GAP-B4-16 | HDR with scaled resolutions | No guide statement ([Displays](https://support.apple.com/guide/mac-help/mh40768/27/mac/27)) | Works on M4, not on M1 Max ([9to5Mac](https://9to5mac.com/2026/10/05/macos-27-just-fixed-the-most-annoying-problem-for-third-party-hdr-displays/)) | Hardware matrix (VP-16) |
| GAP-B4-17 | General > Intel-based Apps | Not in fetched General pages | New section ([Ars Technica](https://arstechnica.com/gadgets/2026/09/macos-27-golden-gate-the-ars-technica-review/), [MacStories](https://www.macstories.net/stories/macos-27-the-macstories-review/13/)) | Live check |
| GAP-B4-18 | AirDrop label | 27 guide: "AirDrop & Continuity" ([AirDrop & Continuity](https://support.apple.com/guide/mac-help/mchl6a407f99/27/mac/27)) | Checklist and MDM ID use "AirDrop & Handoff" (`com.apple.AirDrop-Handoff-Settings.extension`) | Rename not independently confirmed |
| GAP-B4-19 | Keyboard > Dictation | Guide topic [Dictation](https://support.apple.com/guide/mac-help/mh40584/27/mac/27) | Auto-punctuation switch removed ([geeksmodo](https://geeksmodo.com/macos-system-settings/)) | Live check |

## 13.42 LitricOS implications (preliminary)

| ID | Observation | Disposition |
|---|---|---|
| MD.I01 | Sidebar items that depend on hardware and state (Battery vs Energy, Touch ID vs Login Password, VPN, AirPods, Device Management) | ADOPT (concept): build the settings model from capability providers |
| MD.I02 | One extension can provide several sections, and disabling it removes all of them ([Apple](https://developer.apple.com/documentation/devicemanagement/systempreferences)) | ADOPT: a per-section policy key, not only per extension |
| MD.I03 | Undocumented deep-link IDs that break across releases (13 → 27) | AVOID: publish a versioned, documented `litric-settings:` URI contract |
| MD.I04 | Guide labels that drift from the shipping UI (GAP-B4-02/03/04) | AVOID: generate help text from the settings schema |
| MD.I05 | Language-gated pane variants (Siri) | AVOID for core settings; when unavoidable, show why a pane looks different |
| MD.I06 | A single appearance slider for material opacity, defaulting to the middle ([9to5Mac](https://9to5mac.com/2026/09/22/macos-27-gives-you-more-control-over-liquid-glass/)) | INVESTIGATE (G.06, Part 7) |
| MD.I07 | Copying Apple's pane icons, names or layout | AVOID (Rule 10) |

## 13.43 Validation plan additions

| ID | Test | Method | Output tag |
|---|---|---|---|
| VP-13 | Sidebar order and visibility on 26A434 (laptop and desktop) | Read Sidebar.plist and screenshot the sidebar; compare with MD.01 | MEASURED (live) |
| VP-14 | Siri pane variants by language | Switch en-US, en-GB and th-TH; record the controls | MEASURED (live) |
| VP-15 | URL register validity | Open each `macos_settings_urls.csv` row on 27.0.1; record the pane/anchor reached | MEASURED (live) |
| VP-16 | Displays HDR × scaled resolution matrix | M1/M2/M3/M4 with an HDR10 external display | MEASURED (live) |
| VP-17 | Defaults audit | Fresh install; capture each control's state; fill the `Default` column | MEASURED (live) |

## 13.44 Completeness audit (MD)

| ID | Leaf | Status | Basis | Control rows |
|---|---|---|---|---|

| MD.02 | Apple Account | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). | 128 |
| MD.03 | Family | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). | 6 |
| MD.04 | Wi-Fi | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). | 10 |
| MD.05 | Bluetooth | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). | 8 |
| MD.06 | Network | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). | 93 |
| MD.07 | VPN | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). | 18 |
| MD.08 | Battery / Energy | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). | 27 |
| MD.09 | General (About, Software Update, Storage, AppleCare, AirDrop & Handoff, Login Items & Extensions, Language & Region, Date & Time, Sharing, Time Machine, Transfer or Reset, Startup Disk, Device Management) | PARTIAL | 12 of 13 General subpages from the 27 guide; AppleCare & Warranty and AutoFill & Passwords only from support articles (GAP-B4-11). | 153 |
| MD.10 | Accessibility | PARTIAL | 25 Accessibility subpages captured; label conflicts with independent 27 reports (GAP-B4-03, 04). | 322 |
| MD.11 | Appearance | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). | 11 |
| MD.12 | Apple Intelligence & Siri | PARTIAL | Guide documents pane as "Siri"; Apple Intelligence master switch reported removed (GAP-B4-05, 06). | 31 |
| MD.13 | Control Center | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). | 20 |
| MD.14 | Desktop & Dock | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). | 66 |
| MD.15 | Displays | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). | 30 |
| MD.16 | Spotlight | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). | 13 |
| MD.17 | Wallpaper | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). | 12 |
| MD.18 | Screen Saver | PARTIAL | Screen Saver page captured; whether it is a separate sidebar item or reached via Wallpaper is unverified (GAP-B4-09). | 12 |
| MD.19 | Notifications | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). | 8 |
| MD.20 | Sound | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). | 19 |
| MD.21 | Focus | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). | 56 |
| MD.22 | Screen Time | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). | 60 |
| MD.23 | Lock Screen | PARTIAL | Guide lacks the user-name/photo visibility switch reported by geeksmodo (GAP-B4-08). | 8 |
| MD.24 | Privacy & Security | PARTIAL | Privacy list captured; "Accessibility" vs "Device Control and Data Access" conflict (GAP-B4-02). | 83 |
| MD.25 | Touch ID & Password / Login Password | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). | 11 |
| MD.26 | Users & Groups | PARTIAL | Users & Groups captured; legacy login-options topic redirects (GAP-B4-12). | 19 |
| MD.27 | Internet Accounts | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). | 6 |
| MD.28 | Game Center | PARTIAL | Game Center settings now documented in the Games User Guide (1.0 / macOS 27), not the Mac User Guide. | 12 |
| MD.29 | Wallet & Apple Pay | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). | 12 |
| MD.30 | Keyboard | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). | 36 |
| MD.31 | Mouse | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). | 8 |
| MD.32 | Trackpad | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). | 20 |
| MD.33 | Game Controllers | PARTIAL | Only Games User Guide coverage; MDM identifier com.apple.Game-Controller-Settings.extension listed by Apple (GAP-B4-10). | 7 |
| MD.34 | Printers & Scanners | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). | 4 |
| MD.35 | Menu Bar (if separate in 27) | PARTIAL | Menu Bar confirmed as its own sidebar pane in 27 (resolves GAP-006). | 38 |
| MD.01 | Sidebar enumeration (exact order, hardware-dependent items) | PARTIAL | Pane set evidenced from 27 guide navigation paths; canonical sidebar order not published by Apple (GAP-B4-01). | 0 |
| MD.36 | App Settings conventions (Cmd-,) | COMPLETE | Command-Comma and settings-window rules documented by Apple HIG and the Mac shortcuts page; HIG detail continues in Part 7. | 0 |
| MD.37 | x-apple.systempreferences URL schemes | PARTIAL | 209 URL rows; extension IDs listed by Apple for MDM, URL-scheme use undocumented and 27 validity untested (GAP-B4-13). | 0 |
| MD.38 | Configuration profiles / MDM | PARTIAL | SystemPreferences payload (deprecated), Restrictions keys and 27 payload removals captured; DDM replacements only partly covered (GAP-B4-14). | 0 |

## 13.45 Source register statistics (after Part 5)

| Measure | Value |
|---|---|
| Unique URLs | 2,981 (was 1,907 after Part 4) |
| Distinct registrable domains (all statuses) | 270 (was 235) |
| Fully inspected URLs | 826 on 119 domains |
| Primary / independent (inspected) | 575 / 251 |
| Snippet-only (not counted as inspected) | 2,152 |
| Inaccessible | 3 |
| MD-linked URLs | 1078 (151 inspected, on 25 inspected domains) |
| 400-domain target | NOT MET: 270 distinct domains in total, 119 fully inspected |
| Checklist leaves (all) | COMPLETE 16, NOT APPLICABLE 3, NOT FOUND 1, PARTIAL 262, PENDING 248 |

## 13.46 Resume point

Part 6 continues with Batch B5, the macOS 27 desktop model (`MC`), starting at MC.01.
