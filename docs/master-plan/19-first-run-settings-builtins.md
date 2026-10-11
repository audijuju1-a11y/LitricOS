# 19 — First run, settings, and built-ins

## First-run path

`installer → disk/encryption choice → account → locale/input → network → update
policy → accessibility → privacy/AI choices → desktop`. Each step is resumable,
keyboard-accessible, understandable offline, and never claims a security feature
is active until it has verified its prerequisites.

## Settings architecture

Settings are typed, versioned policy objects owned by the relevant service; UI is
not the source of truth. Each setting declares default, scope (system/user/app),
permission, validation, migration, reset, side effect, audit need, rollback and
whether it changes resource/security behaviour.

## v1 built-ins

- Files: sandbox-aware browsing, external media, trash, restore collisions,
  search entry points and safe open/share actions.
- Settings: display, sound, network, Bluetooth, power, accessibility, storage,
  privacy, applications, updates, accounts and diagnostics.
- Search/launcher: apps/settings/files first; content indexing only with scoped
  permission, low priority and battery/pressure pause.
- Browser: Litric browser shell over an upstream engine, with permission and
  profile isolation. Engine development is explicitly out of scope.
- System monitor: process/resource views, battery/thermal health, HCL facts and
  actionable recovery rather than a cosmetic dashboard.

## UX acceptance

Test fresh install, no-network first run, low disk, locale/IME changes, account
lock/logout, invalid setting rollback, external device removal, permission
explanation, high-contrast/text scale and every action from keyboard alone.

