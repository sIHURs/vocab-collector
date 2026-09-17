# Windows database lifecycle validation

Status: **not executed in an isolated Windows environment**. Building NSIS and checking its generated script do not complete this gate.

## Environment

Use a disposable Windows VM/Sandbox or dedicated test account with no real vocabulary data. Copy the exact NSIS package and its SHA256 into that environment. Record Windows version, account type, application versions, hashes and date. Keep the previous package to exercise an actual version upgrade. Do not use the primary development account.

Release-owned directories: `%APPDATA%\app.vocabcollector.desktop.release` and `%LOCALAPPDATA%\app.vocabcollector.desktop.release`. Development data remains under the original `app.vocabcollector.desktop` identifier. External CSV files belong to the user, not the uninstall cleanup.

## Required scenarios

| Scenario | Actions | Expected result | Actual result |
| --- | --- | --- | --- |
| Fresh install | Install, launch, save synthetic multilingual vocabulary and Review results | App starts; release DB only; Check data passes | Pending |
| Upgrade | Install an older release, create data, install newer package; include installer replacement path | Vocabulary, settings and history retained | Pending |
| Same-version reinstall | Reinstall package over itself | Existing data retained | Pending |
| Explicit uninstall | Create release data/cache plus separate development and external CSV sentinels; uninstall through Windows | Release directories removed; development and external files intact | Pending |
| Silent update | Exercise documented `/UPDATE` replacement flow | Data retained; no destructive prompt | Pending |
| Silent uninstall | Run release uninstaller with `/S` in disposable profile | Same cleanup policy; nonzero exit on incomplete cleanup | Pending |
| File in use | Hold a release-owned test file open without delete sharing; uninstall | Remaining data and failure reported; no deletion outside owned roots | Pending |
| Reinstall after uninstall | Install and launch again | Prior synthetic private vocabulary not restored | Pending |
| Native file dialogs | Export CSV and report; cancel; choose existing file; exercise inaccessible destination | Correct cancellation/success/failure, original destination retained on failure | Pending |
| Native performance | Use synthetic 10k/100k-Encounter fixture, measure UI + IPC save/list/check | Compare with recorded layer measurements; record actual full-path latency | Pending |

For each row, record screenshots or terminal output, hashes and data counts before and after. Inspect remaining directories after failure. Never replace a “Pending” result with “Pass” based only on source inspection. Future backups add a separate cleanup case when that feature exists.

## Template maintenance

The vendored NSIS template comes from Tauri CLI 2.11.4 with its MIT license. Local changes always pass `/UPDATE` for installer-driven uninstaller calls, replace the optional data checkbox with an explicit deletion notice, and delegate data removal to the owned-directory hook. Recheck upstream changes and repeat this matrix when updating the Tauri CLI. The legacy pre-release identifier is deliberately not cleaned or migrated.

Release uninstall/install registry keys and the default install directory use the release bundle identifier. Legacy WiX auto-detection is disabled for this first NSIS release, avoiding invocation of a legacy uninstaller whose cleanup policy did not understand `/UPDATE`. Release autostart uses `Vocab Collector Release`; development retains its existing name. These differences must remain intact when refreshing the upstream template.
