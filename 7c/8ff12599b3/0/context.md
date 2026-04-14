# Session Context

## User Prompts

### Prompt 1

A user going through moltis onboarding says:

Getting through onboarding. 
Small notes, let me know if you want it listed on a GH. 
No LM Studio/llama.cpp server offered on llm chosing step.

### Prompt 2

local llama.cpp models with hugging face downloads should definitely be included by default on the onboarding LLM. It's a great moltis feature (local llm).

### Prompt 3

commit and push

### Prompt 4

Another one, I had this review comment:

@krsyoung
krsyoung
(Christopher Young)
yesterday
@penso totat nit, the password placeholder is still showing as "At least 8 characters":

moltis/crates/web/src/assets/js/locales/en/settings.js
Line 102 in ea9fc8d
 passwordPlaceholder: "At least 8 characters", 
Image
There were a few others when searching the code too ... not sure if they should also get with the times and move to 12?

https://github.com/search?q=repo%3Amoltis-org%2Fmoltis%208%20charact...

### Prompt 5

INFO audit: zizmor: 🌈 completed ./.github/workflows/release.yml
No findings to report. Good job! (28 suppressed)
[local/zizmor] passed in 1s
Diff in /Users/penso/tmp/molt/moltis/crates/tools/src/sandbox/tests/core.rs:31:
         .iter()
         .position(|a| a == "--hostname")
         .expect("--hostname flag missing");
-    assert_eq!(args[hostname_pos + 1], "sandbox", "--hostname value should be 'sandbox'");
+    assert_eq!(
+        args[hostname_pos + 1],
+        "sandbox",
+        "...

### Prompt 6

ok: deb arch: aarch64 → arm64
Cross-validating against release.yml...
🌈 zizmor v1.22.0
  ok: release.yml: no --deb-revision flag (consistent with install.sh)
  ok: release.yml: uses --deb-version for .deb naming
i18n parity OK: 3 locales, 18 namespaces.
  ok: release.yml: AppImage naming matches install.sh
Checking install.sh sync...
  ok: install.sh and website/install.sh are identical

All 15 install package name checks passed
[local/i18n] passed in 0s
[local/install-names] passed in 0s
cra...

### Prompt 7

Checking release_tag() logic...
  ok: release_tag('20260327.05') = '20260327.05' (date-based, bare)
  ok: release_tag('0.1.3') = 'v0.1.3' (semver, v-prefixed)
Checking architecture mappings...
i18n parity OK: 3 locales, 18 namespaces.
  ok: deb arch: x86_64 → amd64
  ok: deb arch: aarch64 → arm64
Cross-validating against release.yml...
crates/web/src/assets/js/onboarding-view.js format ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[local/i18...

