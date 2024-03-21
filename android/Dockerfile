FROM rust:1.76

RUN apt-get update && apt-get install android-sdk -yy
RUN mkdir /usr/lib/android-sdk/cmdline-tools/
WORKDIR /usr/lib/android-sdk/cmdline-tools
RUN wget https://dl.google.com/android/repository/commandlinetools-linux-9477386_latest.zip
RUN unzip commandlinetools-linux-9477386_latest.zip
RUN mv /usr/lib/android-sdk/cmdline-tools/cmdline-tools/ /usr/lib/android-sdk/cmdline-tools/latest/
RUN yes | /usr/lib/android-sdk/cmdline-tools/latest/bin/sdkmanager --licenses
RUN /usr/lib/android-sdk/cmdline-tools/latest/bin/sdkmanager --install "ndk;26.2.11394342" --channel=0
RUN /usr/lib/android-sdk/cmdline-tools/latest/bin/sdkmanager "platform-tools" "platforms;android-31"
RUN /usr/lib/android-sdk/cmdline-tools/latest/bin/sdkmanager "platform-tools" "platforms;android-33"
RUN /usr/lib/android-sdk/cmdline-tools/latest/bin/sdkmanager "platform-tools" "platforms;android-34"

RUN rustup update
RUN rustup target add aarch64-linux-android
RUN rustup target add armv7-linux-androideabi
RUN cargo install cargo-apk

RUN apt-get update && apt-get install zip

ENV ANDROID_HOME=/usr/lib/android-sdk
ENV ANDROID_NDK_ROOT=/usr/lib/android-sdk/ndk/26.2.11394342

WORKDIR /app/

# RUN cargo apk build --release