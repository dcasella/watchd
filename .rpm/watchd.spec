%define __spec_install_post %{nil}
%define __os_install_post %{_dbpath}/brp-compress
%define debug_package %{nil}
%define _unitdir /usr/lib/systemd/system
%define _configdir /etc/watchd

Name: watchd
Summary: A fearlessly concurrent filesystem watcher daemon
Version: @@VERSION@@
Release: 1
License: MIT
Group: System Environment/Daemons

BuildRoot: %{_tmppath}/%{name}-%{version}-%{release}-root

Source0: %{name}-%{version}.tar.gz

Requires(post): systemd
Requires(preun): systemd
Requires(postun): systemd

%description
%{summary}

%prep
%setup -q

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
install -d %{buildroot}/usr/sbin
install -d %{buildroot}%{_configdir}
install -d %{buildroot}%{_unitdir}
install -m0755 usr/sbin/watchd %{buildroot}/usr/sbin
install -m0640 etc/watchd/config.toml %{buildroot}%{_configdir}
install -m0644 usr/lib/systemd/system/watchd.service %{buildroot}%{_unitdir}

%clean
rm -rf %{buildroot}

%post
%systemd_post watchd.service

%preun
%systemd_preun watchd.service

%postun
%systemd_postun_with_restart watchd.service

%config
%{_configdir}/config.toml

%files
%defattr(-,root,root)
%{_sbindir}/*
%{_configdir}
%{_unitdir}/watchd.service
