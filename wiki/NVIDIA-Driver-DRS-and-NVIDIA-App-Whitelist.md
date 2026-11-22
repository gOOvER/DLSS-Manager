# 🎛️ NVIDIA Treiber-DRS (`nvapi64.dll`) & NVIDIA App Whitelist

Eine der mächtigsten Funktionen von **DLSS Manager** ist die direkte Steuerung der **NVIDIA Driver Settings (DRS)** über `nvapi64.dll` in Kombination mit dem **NVIDIA App Override-Whitelist-Unlocker**.

---

## 1. Wie funktioniert NVIDIA Treiber-DRS (`nvapi64.dll`)?

Anstatt inoffizielle Wrapper-DLLs in den Spielordner zu injizieren (was von Anti-Cheat-Systemen wie EasyAntiCheat blockiert wird), spricht DLSS Manager direkt die offizielle Treiber-Schnittstelle `nvapi64.dll` von NVIDIA an.

### Verwendete NVAPI DRS Setting-IDs

| Funktion | NVAPI Setting-ID (Hex) | Werte |
| :--- | :--- | :--- |
| **DLSS-SR Override aktivieren** | `0x10E41DF3` | `1` = Aktiv, *gelöscht* bei `Default` (`0` bedeutet im Treiber „Durch NVIDIA App gesperrt“!) |
| **DLSS-SR Render-Preset** | `0x10E41DF7` | `0x01`–`0x06` (`Preset A–F`), `0x0A` (`Preset J`), `0x0B` (`Preset K`), `0x0C` (`Preset L`), `0x0D` (`Preset M`), `0x00FFFFFF` (`Latest`) |
| **DLSS-RR Override aktivieren** | `0x10E41DF5` | `1` = Aktiv, *gelöscht* bei `Default` |
| **DLSS-RR Render-Preset** | `0x10E41DF8` | `0x04` (`Preset D`), `0x06` (`Preset F`), `0x00FFFFFF` (`Latest`) |
| **DLSS-FG Override aktivieren** | `0x10E41DF4` | `1` = Aktiv, *gelöscht* bei `Default` |
| **DLSS-FG Render-Preset** | `0x10E41DF6` | `0x01` (`Preset A`), `0x02` (`Preset B`), `0x00FFFFFF` (`Latest`) |
| **Streamline Override Gate** | `0x10E41E06` | Wird von der NVIDIA App als Sperr-Gate (`0`) missbraucht; wird von DLSS Manager automatisch bereinigt |
| **DLSS-G Multi-Frame Gen (MFG)** | `0x10308298`, `0x104D6667`, `0x10562D0F`, `0x10CF4125` | Steuert MFG-Modus (`Default`, `Fixed`, `Dynamic`, `Off`), Multiplikator (`2×–6×`) und Dynamic Ziel-FPS |

### Wichtig: Warum `DeleteProfileSetting` statt `0` bei „Default“?
Im NVIDIA-Treiber bedeutet der Wert `0` bei `0x10E41DF3` **nicht** „Standard“, sondern **„DLSS-Override explizit für dieses Spiel verbieten“**!  
Wenn du in DLSS Manager ein Preset auf `Default` zurücksetzt, ruft DLSS Manager deshalb sauber `NvAPI_DRS_DeleteProfileSetting` auf. Dadurch wird die Sperre komplett aus dem Spielprofil gelöscht, sodass globale Presets oder die Spieleinstellung wieder normal greifen.

---

## 2. Warum überschreibt die NVIDIA App Presets wieder auf `0`?

Wenn die **NVIDIA App** auf deinem PC installiert ist, lädt deren Hintergrunddienst (`NVDisplay.ContainerLocalSystem`) regelmäßig eine interne Whitelist:

1. **`ApplicationStorage.json`** unter:
   ```text
   C:\ProgramData\NVIDIA Corporation\NVIDIA app\UpdateFramework\ota-artifacts\grd\post-processing\*\ApplicationStorage.json
   ```
2. **`fingerprint.db`** (Chromium/CEF-Cache der NVIDIA App) unter:
   ```text
   %LOCALAPPDATA%\NVIDIA Corporation\NVIDIA app\CefCache\Default\fingerprint.db
   ```

Für Spiele, die von NVIDIA noch nicht offiziell für die NVIDIA-App-Oberfläche freigeschaltet wurden (wie z. B. **Star Citizen**), stehen in dieser Liste standardmäßig folgende Sperr-Flags:
- `"Disable_SR_Override": true`
- `"Disable_RR_Override": true`
- `"Disable_FG_Override": true`
- `"Disable_SR_Model_Override": true`
- `"Disable_RR_Model_Override": true`

Sobald die NVIDIA App im Hintergrund startet oder synchronisiert, liest sie diese `true`-Flags und schreibt aktiv **`0x10E41DF3 = 0`** (Blockiert!) in das NVIDIA-Treiberprofil des Spiels!

---

## 3. Die Lösung: 1-Klick NVIDIA App Override-Whitelist in DLSS Manager

DLSS Manager löst dieses Problem auf drei Ebenen:

### Ebene 1: 1-Klick Whitelist-Unlocker (`🔓 Alle Spiele freischalten`)
Auf dem **Hub** (und unter **⚙️ Einstellungen → Globale NVIDIA DLSS-Presets**) prüft DLSS Manager beim Start automatisch den Status deiner NVIDIA App:
- Ein Klick auf **„🔓 Alle Spiele freischalten“** legt automatisch ein Backup (`ApplicationStorage.json.dlss_manager.bak` & `fingerprint.db.dlss_manager.bak`) an.
- Alle `Disable_FG_Override`, `Disable_RR_Override`, `Disable_SR_Override`, `Disable_RR_Model_Override` und `Disable_SR_Model_Override`-Einträge werden in `ApplicationStorage.json` und `fingerprint.db` auf `false` (`0`) gesetzt und `IsOpsSupported` aktiviert.
- Über **„🔄 NVIDIA Dienst neu starten“** wird der Hintergrunddienst neu geladen, sodass die NVIDIA App kein einziges Spiel mehr zurücksetzt.

### Ebene 2: Automatische Bereinigung von `0`-Blockaden im Treiberprofil
Selbst wenn du ein Spiel auf `Default` (Globales Preset erben) belässt, prüft DLSS Manager beim Start (`clear_game_dlss_profile_blocks`), ob die NVIDIA App heimlich eine `0`-Sperre in das Spielprofil (z. B. `"Squadron 42 - Star Citizen"`) geschrieben hat, und löscht diese Sperre automatisch aus dem Treiber.

### Ebene 3: Live-DRS-Readback im Spieldetails-Modal (`Treiber-Live`)
Wenn du ein Spiel in DLSS Manager anklickst, liest DLSS Manager per `NvAPI_DRS_GetSetting` in Echtzeit den tatsächlichen Zustand aus dem NVIDIA-Treiber aus und zeigt ein Live-Badge an:
- **`🟢 Treiber-Live: Preset K`** → Das Preset ist aktiv und im NVIDIA-Treiber verifiziert.
- **`🟠 ⚠️ Treiber-DRS: Durch NVIDIA App blockiert (0)`** → Die NVIDIA App hat eine `0`-Sperre gesetzt (ein Klick auf **🔓 Alle Spiele freischalten** oder erneutes Auswählen des Presets behebt dies sofort).

---

## 4. Lokaler NVIDIA NGX OTA-Modell-Cache

Moderne NVIDIA-Treiber laden aktualisierte DLSS-4.5-Modelle automatisch über NVIDIAs **NGX Over-The-Air (OTA)**-System herunter und legen sie verschlüsselt/signiert hier ab:
```text
C:\ProgramData\NVIDIA\NGX\models\dlss\versions\
```
DLSS Manager zeigt die höchste lokal installierte **NGX OTA-Version** (z. B. `v310.5.3` / `v310.9.16`) direkt im NVIDIA-App-Statusbanner auf dem Hub an. Dank dieses NGX-Caches kann der NVIDIA-Treiber selbst bei Spielen mit älterer DLSS-4.0-DLL auf der Festplatte (wie Star Citizen mit `v310.2.1.0`) zur Laufzeit das neuere Transformer-Gen-2-Modell (**Preset K**) einspeisen, sobald das Treiber-DRS-Override aktiv ist!

---

## 5. Überprüfung im Spiel: Das DLSS On-Screen Overlay

Du möchtest im Spiel mit eigenen Augen sehen, ob **Preset K** und **DLSS 4.5** laufen?
1. Aktiviere oben rechts in der Topbar von DLSS Manager den Schalter **„DLSS-Overlay“** (setzt `HKLM\SOFTWARE\NVIDIA Corporation\Global\NGXCore\ShowDlssIndicator = 0x400`).
2. Starte dein Spiel mit aktiviertem DLSS.
3. Unten links im Spielbildschirm blendet NVIDIA nun einen kleinen Textblock ein, der die aktive **DLSS-DLL-Version**, die **Render-Auflösung** und das aktive **Preset (`Preset K`, `Transformer`)** live anzeigt.
4. Schalte den Schalter nach der Kontrolle einfach wieder aus.
