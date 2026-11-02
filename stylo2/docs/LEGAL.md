# Legal & interoperability notes (EU / Netherlands)

**This is general, informational context — not legal advice.** It sketches the
framework a Netherlands-based developer would reason within when building
independent interoperable software for hardware they own. For commercial
distribution, get advice from a qualified lawyer. Citations are to the
instruments by name; verify current consolidated texts and national case law.

## Starting point

The developer owns the OnePlus Pad 3 and the Stylo 2, and the goal is
**independent development of software that interoperates** with hardware already
owned and operated — making the pen's existing haptic capability usable by the
owner's own apps. That is an interoperability objective, not an attempt to copy
or redistribute OnePlus's software.

## Rights that generally support this work

- **Software interoperability — EU Directive 2009/24/EC (Software Directive).**
  - *Art. 5(3)*: a lawful user may observe, study and test the functioning of a
    program to determine the ideas and principles underlying any element
    (e.g. by watching its behaviour, inspecting interfaces), without the
    rightholder's authorisation.
  - *Art. 6*: decompilation is permitted **where indispensable to obtain
    interoperability** of an independently created program, subject to
    conditions (done by/for a lawful user; the interface information is not
    already readily available; confined to the parts necessary; not used to
    create a substantially similar competing program or for other acts
    infringing copyright).
  - *Art. 8*: contract terms contrary to Art. 6 (and the Art. 5(3) exception)
    are null and void — a EULA generally cannot contract these interoperability
    rights away.
  - **Dutch implementation:** Auteurswet (Copyright Act) arts. 45m (and 45k/45l)
    transpose these into national law.
- **Use of the device's own documented Bluetooth capabilities** and **publicly
  available SDKs/documentation** is ordinary use, subject to the licence terms
  that actually accompany those materials.
- **Consumer law.** Dutch Burgerlijk Wetboek and EU consumer-protection rules
  (incl. the Digital Content Directive 2019/770 as implemented in NL) support an
  owner's normal use of purchased goods; unreasonably onerous standard terms can
  be open to challenge. (General principle, fact-specific.)

## Lines this project deliberately stays on the safe side of

| Activity | This project |
| --- | --- |
| Independent development of interoperable software | **Yes** — a clean-room-style portable core + backends, no OnePlus code copied |
| Analysing interfaces for compatibility | **Yes** — behavioural analysis and the already-public AIDL *shape*; any decompilation would be only as far as Art. 6 allows and against the owner's own device |
| Using public SDKs/docs/Bluetooth per their terms | **Yes**, where they exist |
| **Circumventing an effective technical protection / access control** | **No.** The restricted permission is an access control; the project does **not** bypass it — it reports denial honestly. The root/priv-app mechanism that *does* alter system access control is explicitly excluded |
| **Reusing proprietary credentials or cryptographic secrets** (vendor signing keys, tokens) | **No.** None are extracted, bundled, or required by this code |

The distinction between **circumvention of a technical protection measure**
(governed more restrictively — InfoSoc Directive 2001/29/EC Art. 6; Auteurswet
Art. 29a) and **legitimate interoperability analysis** (Software Directive) is
the crux. By declining the privileged-install route and by refusing to spoof an
authorised identity — reporting `PermissionDenied`/`Unauthorized` instead — the
project keeps to the interoperability side of that line.

## Practical guidance embedded in the design

- **Pursuing the official SDK route** (B-feasibility) means operating *within*
  OnePlus's developer terms — authorised access, not circumvention. Preferred
  where available.
- **BLE reconnaissance** (C-protocol) is analysis of the communications of the
  owner's **own** device — squarely the Art. 5(3) "observe/study/test" activity.
  It does not involve breaking encryption or reusing secrets.
- **Own signing certificate** only; **no** use of another developer's identity
  or private keys.

## What this document does *not* claim

- It does **not** conclude that any specific act is lawful in a given situation —
  that depends on facts, the actual licence terms, and current case law.
- It does **not** treat hardware ownership as granting rights over OnePlus's
  proprietary software, credentials, or protected source.
- Equally, it does **not** treat the absence of a published SDK as a prohibition
  on independent interoperability development.
