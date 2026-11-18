Name:           velox-transcode
Version:        %{velox_version}
Release:        1%{?dist}
Summary:        Transcodificador nativo de vídeo em lote
# Artefato para verificação local. A licença do aplicativo será definida antes da distribuição pública.
License:        LicenseRef-Unspecified
Source0:        velox-rpm-payload.tar.gz
Requires:       /usr/bin/ffmpeg
Requires:       /usr/bin/ffprobe
Requires:       qt6-qtwayland
Requires:       xdg-desktop-portal
Requires:       desktop-file-utils

%description
Velox Transcode é um aplicativo nativo para converter vídeos em lote para H.264 e Apple ProRes.

%prep
%setup -q -c -n velox-rpm-payload

%install
mkdir -p "%{buildroot}"
cp -a usr "%{buildroot}/"

%files
%{_bindir}/velox-transcode
%{_datadir}/applications/dev.veloxtranscode.Velox.desktop
%{_datadir}/icons/hicolor/scalable/apps/dev.veloxtranscode.Velox.svg
%{_datadir}/icons/hicolor/*/apps/dev.veloxtranscode.Velox.png
%license %{_licensedir}/velox-transcode/
