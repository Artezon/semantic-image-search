## Work in progress

To try it out, download from the link below. AI model that is needed for this app to work is bundled.<br/>
https://drive.google.com/drive/folders/1N93bLdlH7F-Ukzs0XBieRbSYU19azzwm (~2 GB)

Windows - supported<br>
Linux - soon<br>
MacOS - soon, maybe (if I get my hands on a mac)<br>

Currently only MetaCLIP 2 model is supported

<img src="https://github.com/user-attachments/assets/4afdc3c6-399a-480f-997c-d6d14fb54fc9" />
<p></p>
<img src="https://github.com/user-attachments/assets/3b8597a1-754d-47f3-bb63-e7c81c222729" />
<p></p>
<img src="https://github.com/user-attachments/assets/afe4ead9-6340-4261-925e-6d35b5fa6253" />
<p></p>
<img src="https://github.com/user-attachments/assets/7e485d41-94c9-491f-a414-41611f0a3b3a" />
<p></p>
<img src="https://github.com/user-attachments/assets/4ddefa7a-b2df-4aec-b8ad-3e07b4617b2f" />

### To compile with advanced image formats support (HEIC, HEIF, AVIF):
Install vcpkg, e.g. to `C:\vcpkg`:
```
git clone https://github.com/microsoft/vcpkg.git
cd vcpkg
.\bootstrap-vcpkg.bat -disableMetrics
vcpkg integrate install
```
Install those libraries:
```
vcpkg install libheif:x64-windows-static dav1d:x64-windows-static
```
Edit paths in `.cargo/config.toml` to match your vcpkg installation accordingly.
