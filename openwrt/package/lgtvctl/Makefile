include $(TOPDIR)/rules.mk

PKG_NAME:=lgtvctl
PKG_VERSION:=0.6.0
PKG_RELEASE:=1

PKG_SOURCE_PROTO:=git
PKG_SOURCE_URL:=https://github.com/Fancier-tech/lg-webos-openwrt.git
PKG_SOURCE_VERSION:=main
PKG_MIRROR_HASH:=skip

PKG_BUILD_DEPENDS:=rust/host openssl
PKG_BUILD_FLAGS:=no-mips16

RUST_PKG_LOCKED:=0
RUST_PKG_FEATURES:=tls-native

include $(INCLUDE_DIR)/package.mk
include $(TOPDIR)/feeds/packages/lang/rust/rust-package.mk

CARGO_PKG_ARGS += --no-default-features

define Package/lgtvctl
  SECTION:=utils
  CATEGORY:=Utilities
  TITLE:=LG webOS TV controller and local HTTP API
  URL:=https://github.com/Fancier-tech/lg-webos-openwrt
  DEPENDS:=+libgcc +libopenssl
endef

define Package/lgtvctl/description
  lgtvctl controls LG webOS TVs from OpenWrt. It supports Wake-on-LAN,
  webOS WSS commands and a local HTTP API for Alice/smart-home automations.
endef

define Package/lgtvctl/conffiles
/etc/lgtvctl.toml
endef

define Package/lgtvctl/install
	$(INSTALL_DIR) $(1)/usr/bin
	$(INSTALL_BIN) $(PKG_INSTALL_DIR)/bin/lgtvctl $(1)/usr/bin/lgtvctl
	$(INSTALL_DIR) $(1)/etc/init.d
	$(INSTALL_BIN) ./files/lgtvctl.init $(1)/etc/init.d/lgtvctl
	$(INSTALL_DIR) $(1)/etc
	$(INSTALL_CONF) ./files/lgtvctl.toml $(1)/etc/lgtvctl.toml
endef

$(eval $(call BuildPackage,lgtvctl))
