FROM archlinux:latest

# Update system and install dependencies
RUN pacman -Syu --noconfirm && \
    pacman -S --noconfirm \
      base-devel \
      rust \
      cargo \
      pkg-config \
      git \
      curl \
      file \
      fakeroot \
      openssl \
      xz \
      rsync \
      fontconfig \
      freetype2 \
      libx11 \
      libxft \
      glib2 \
      cairo \
      pango \
      gdk-pixbuf2 \
      atk \
      at-spi2-core \
      at-spi2-atk \
      gtk3 \
      gtk4 \
      libadwaita \
      && pacman -Scc --noconfirm

# Create non-root user for makepkg
RUN useradd -m -G wheel builder && \
    echo '%wheel ALL=(ALL) NOPASSWD: ALL' >> /etc/sudoers

# Set up environment
USER builder
WORKDIR /home/builder
ENV PATH="/home/builder/.cargo/bin:${PATH}"

# Verify installation
RUN rustc --version && cargo --version

CMD ["/bin/bash"]
