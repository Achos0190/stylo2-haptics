# Stylo2 Haptics PoC — Route A (Oplus binder)

The **smallest third-party app** that tries to make the Stylo 2 vibrate by
binding the stock `com.oplus.ipemanager` accessory service and calling its
haptic methods. One run on the Pad 3 is simultaneously:

- the **proof of concept** — if the actuator fires, this app *is* the working
  PoC, with no reverse-engineering; and
- the **isolating experiment** — it reports discovery → binding → authorization
  → execution separately, so a failure tells you *which* trust boundary blocks
  (see `../../stylo2/docs/H-trust-boundaries.md`).

It needs **no root**. The existing project used root only to *grant* the
`com.oplus.permission.safe.IOT` permission; this app does not assume the
permission is granted — it attempts the bind regardless and records exactly
what the platform does. That unprivileged attempt is the fast, decisive test the
existing Magisk-based project never ran cleanly.

> **Not built in the authoring environment** (no Android SDK there). It is a
> complete, dependency-light standalone Gradle project (plain `Activity` +
> `android.widget`, no androidx/Compose). Build it on your machine.

## Build & install

```bash
cd android/stylo2-poc
./gradlew :app:installDebug      # or :app:assembleDebug then adb install
```

Install it like any app (sideload / `adb install`) — do **not** place it in
`/system` and do **not** use Magisk; the whole point is the unprivileged test.

## Run

1. Make sure the Stylo 2 is paired and working (draw in Notes once).
2. Open **Stylo2 Haptics PoC**.
3. Tap **1. Bind / probe service** — read the log.
4. Pick a profile, tap **2. Trigger haptic** — **watch the pen**.
5. Tap **3. Stop feedback**.

## Reading the result

| What the log shows | Meaning | Next |
| --- | --- | --- |
| Bind ok, AUTHZ accepted, **pen vibrates** on trigger | **DONE** — Route A works unprivileged; this is the PoC | profile selection, timing, production integration |
| `SecurityException`/`bindService returned false` at BIND | B1: the IOT permission is enforced and not granted to an ordinary app | go to Route B (direct BLE) — `../../stylo2/docs/J-capture-plan.md` |
| Bind ok but AUTHZ query throws | B2: the service validates the caller (UID/package/signature) after bind | capture what Notes does (Route B); consider official-SDK authorisation |
| Bind + AUTHZ ok, calls return, but **no vibration** | B4: the protocol/session, not Android, gates the motor | inspect the session in an HCI snoop (Route B) |
| `version = NOT INSTALLED` or 0 discovery matches | the service isn't present/queryable on this build | re-check the firmware; reconsider Route B |

## Required evidence to report (mission §"Required evidence")

Copy the on-screen log and fill in:

- **Inspected:** this PoC on <firmware build>, <date>.
- **Observed:** paste the log (ENV / DISCOVERY / BIND / AUTHZ / EXEC lines).
- **Physical result:** did the pen vibrate on **2. Trigger**? yes / no.
- **Uncertain:** anything the log didn't resolve.
- **Root/privileged needed:** no (this run) — note if a route later requires it.
- **Tested on device:** yes/no, which device + build.

Do not record a "works" unless the pen physically vibrated.

## Boundaries

No root, no `/system` install, no firmware changes. This app only talks to a
system service that is already present; it sends no Bluetooth itself. If BIND or
AUTHZ is refused, that is the finding — report it; do not try to escalate
privilege. The reproduced AIDL is the repo's community reproduction; if the
descriptor line doesn't match, that is itself evidence (the interface differs on
this firmware) — capture it and verify against the installed APK with JADX.
