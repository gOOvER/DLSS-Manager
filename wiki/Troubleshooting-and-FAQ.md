# ❓ Fehlerbehebung & Häufige Fragen (FAQ)

Hier findest du direkte Antworten und Lösungen auf die häufigsten Fragen rund um **DLSS Manager**, **NVIDIA Treiber-DRS Presets**, die **NVIDIA App** und den **Spiele-Scanner**.

---

### 1. Werden meine DLSS-Presets von der NVIDIA App wieder überschrieben?
**Ursache:** Ja, standardmäßig setzt der Hintergrunddienst der **NVIDIA App** (`NVDisplay.ContainerLocalSystem`) bei jedem Start oder Update alle Spiele, die in NVIDIAs interner Datei `ApplicationStorage.json` bzw. `fingerprint.db` mit `"Disable_SR_Override": true` markiert sind (wie z. B. **Star Citizen**), im Treiberprofil aktiv auf `0` (Blockiert) zurück.  
**Lösung in DLSS Manager:**
1. Klicke auf dem **Hub** (oder unter **⚙️ Einstellungen → Globale NVIDIA DLSS-Presets**) im Bereich **„🛡️ NVIDIA App Override-Status“** auf **„🔓 Alle Spiele freischalten“**.
2. Klicke anschließend auf **„🔄 NVIDIA Dienst neu starten“**.
3. Zudem prüft und bereinigt DLSS Manager bei jedem App-Start automatisch alle `0`-Blockaden in deinen Spielprofilen und setzt deine gespeicherten Presets erneut im Treiber durch.

---

### 2. Wie kann ich sicher prüfen, welches DLSS-Preset im Spiel tatsächlich aktiv ist?
Du hast zwei Möglichkeiten:
1. **In DLSS Manager (Live-Treiber-Readback):** Öffne die Details eines Spiels. In der Box **„🎛️ Angewendete NVIDIA DLSS Presets (Treiber-DRS)“** liest DLSS Manager den echten Live-Wert aus `nvapi64.dll` aus und zeigt z. B. **`🟢 Treiber-Live: Preset K`** an.
2. **Direkt im Spiel (DLSS-Overlay):** Schalte oben rechts in der Topbar von DLSS Manager den Schalter **„DLSS-Overlay“** ein und starte das Spiel. Unten links im Spielbildschirm zeigt das offizielle NVIDIA-NGX-Overlay die exakte DLL-Version und das aktive Preset (`Preset K`, `Transformer`) an.

---

### 3. Ein Spiel (z. B. aus Ubisoft Connect, EA App oder auf einer anderen Festplatte) fehlt in der Liste – warum?
- **Werden alle Festplatten gescannt?** Ja, DLSS Manager scannt automatisch alle Laufwerksbuchstaben von `A:\` bis `Z:\` nach Steam, Star Citizen, Battle.net, Epic Games, GOG, EA App, Ubisoft Connect, Xbox Game Pass sowie allgemeinen Spieleordnern (`Games`, `Spiele`, `PC Games`).
- **Wichtig:** Ein Spiel erscheint nur dann in der Liste, wenn im Spielordner mindestens eine unterstützte **64-Bit Upscaler-/Streaming-DLL** (`nvngx_dlss*.dll`, `sl.*.dll`, `libxess*.dll` oder `dstorage*.dll`) gefunden wird.
- **Tipp:** Ältere Spiele (wie z. B. *Tom Clancy's The Division 1* von 2016), die vor der Einführung von DLSS/XeSS/DirectStorage erschienen sind, besitzen keine solchen DLLs und werden daher nicht als Upscaler-Spiele gelistet.
- Falls du ein Spiel in einem ungewöhnlichen Pfad installiert hast, füge den Ordner einfach unter **⚙️ Einstellungen → Eigene Spieleordner** hinzu und klicke links auf **„🔍 Spiele scannen“**.

---

### 4. Warum wurde ein Spiel doppelt (z. B. unter Steam und Epic Games) angezeigt, obwohl es auf Epic deinstalliert ist?
- Manche Deinstallationsprogramme lassen alte Restordner mit einem `.egstore`-Unterordner (Epic) oder Restdateien in `steamapps\common\` bzw. Battle.net zurück.
- **Lösung (automatisch aktiv):** DLSS Manager verifiziert jeden Epic-Games-Ordner gegen die offiziellen Epic-Manifeste (`C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests\*.item` mit `bIsIncompleteInstall != true`), prüft Steam-Ordner gegen `appmanifest_*.acf`, verifiziert Battle.net-Spiele gegen aktive Flavor-Executables (`_retail_`, `_classic_`) und verwirft generische Standalone-Duplikate, wenn dasselbe Spiel bereits unter einem offiziellen Launcher installiert ist.

---

### 5. Warum kann ich bei Star Citizen die DLL nicht direkt per „⚡ Update“ ersetzen?
- **Star Citizen** nutzt **EasyAntiCheat (EAC)**, das beim Start den exakten Datei-Hash von `Bin64\nvngx_dlss.dll` prüft. Würde die Datei auf der Festplatte ausgetauscht, würde EAC den Spielstart blockieren.
- Da Star Citizen bereits mit DLSS 4.0 (`v310.2.1.0`) ausgeliefert wird, zeigt DLSS Manager dort **„🛡️ Nur Treiber-DRS“** an: Du lässt die Original-DLL im Spielordner unangetastet und aktivierst stattdessen **Preset K (Transformer Gen 2)** sicher über **„✨ Optimales Preset anwenden (Preset K)“** im NVIDIA-Treiber!

---

### 6. Ein Spiel stürzt nach einem DLL-Update ab oder zeigt Grafikfehler – wie stelle ich das Original wieder her?
DLSS Manager legt vor **jedem** Überschreiben einer DLL automatisch ein Backup der Originaldatei unter `%LOCALAPPDATA%\gOOvER\DLSS-Manager\backups\` an:
- **Einzelne Technologie zurücksetzen:** Öffne das Spiel in DLSS Manager und klicke in der Zeile der jeweiligen Technologie (z. B. `DLSS` oder `Streamline`) auf **„⏪ Backup“**.
- **Komplettes Spiel zurücksetzen:** Klicke unten im Spieldetails-Modal auf **„⏪ Alle Backups wiederherstellen“** oder wechsle links in den Reiter **🛡️ Backups** und klicke beim Spiel auf **„⏪ Original wiederherstellen“**.

---

### 7. Werden AMD FSR DLLs (`amd_fidelityfx_*.dll`) auch verwaltet?
Nein. Auf Wunsch und zugunsten eines reinen Fokus auf **NVIDIA DLSS 4.5 / 5**, **NVIDIA Streamline**, **Intel XeSS 2** und **Microsoft DirectStorage** aus offiziellen Hersteller-Upstreams wurde AMD FSR ab `v1.0.0-beta.2` vollständig aus dem Scanner und Update-Katalog entfernt.
