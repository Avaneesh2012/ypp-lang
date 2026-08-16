#!/bin/bash
echo "========================================================"
echo "              Y++ Language 1.0 Installer"
echo "========================================================"
echo ""
echo "Compiling Y++ Engine and IDE..."
mkdir -p bin
javac -encoding UTF-8 -d bin src/*.java

if [ $? -ne 0 ]; then
    echo ""
    echo "[ERROR] Java compiler (javac) failed or JDK 17+ is missing."
    exit 1
fi

echo "Compilation successful!"
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

cat << EOF > ypp
#!/bin/bash
java -cp "$DIR/bin" ypp.Ypp "\$@"
EOF
chmod +x ypp

cat << EOF > ypp-ide
#!/bin/bash
java -cp "$DIR/bin" ypp.YppIDE "\$@" &
EOF
chmod +x ypp-ide

echo "========================================================"
echo "  Y++ 1.0 Installed Successfully!"
echo ""
echo "  - To launch Y++ IDE: Run './ypp-ide'"
echo "  - To run Y++ CLI: Run './ypp myfile.ypp'"
echo "========================================================"
