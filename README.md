### Welcome

Learning rust. I have no idea what I'm doing

# Running list of setup commands

# For WSL

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
sudo apt install -y build-essential curl git pkg-config libssl-dev
rustup target add x86_64-pc-windows-gnu
sudo apt install mingw-w64 -y
sudo apt install mingw-w64-tools

cp /mnt/c/Windows/System32/vulkan-1.dll ~/winlibs/vulkan/
cd ~/winlibs/vulkan

### Generate DEF file
gendef vulkan-1.dll

### Produce libvulkan-1.a
x86_64-w64-mingw32-dlltool \
    -d vulkan-1.def \
    -l libvulkan-1.a