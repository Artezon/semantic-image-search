## Work in progress

To try it out, download here (provided AI model is needed for this app to work)
- Without CUDA support: https://drive.google.com/drive/folders/1VHO1Z3KKU0QYjBmSLoATshQYAd8DJ81e (~2 GB)
- With CUDA support: https://drive.google.com/drive/folders/1lXXM110s2xlLiaUip_S_hWCdYFgoormH (~3 GB)

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

### To compile with HEIF support:
Install vcpkg to `C:\vcpkg`:
```
git clone https://github.com/microsoft/vcpkg.git
cd vcpkg
.\bootstrap-vcpkg.bat -disableMetrics
vcpkg integrate install
winget install LLVM.LLVM
[Environment]::SetEnvironmentVariable("VCPKG_ROOT", "C:\vcpkg", "User")
$env:PATH = "$env:VCPKG_ROOT;$env:PATH"
```
Install those libraries:
```
vcpkg install libheif:x64-windows-static
```
