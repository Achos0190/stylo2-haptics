MODDIR=${0%/*}
MODULE_DIR="/sys/module"
ANY_DIR="/dev/anyfs"

bind_mount(){
    source="${2:-$MODDIR/$1}"
    mount --bind "$source" "$1"
}

bind_mount_recursive() {
    for file in "$MODDIR/$1"/*; do
        sub_item=$(basename "$file")
        if [ -f "$file" ]; then
            bind_mount "$1/$sub_item" "$file"
        elif [ -d "$file" ]; then
            mkdir -p "$1/$sub_item"
            bind_mount_recursive "$1/$sub_item"
        fi
    done
}

overlay_mount(){
    [ -d $ANY_DIR/upper/"$1" ] && remount="remount,"
    mkdir -p $ANY_DIR/upper/"$1" $ANY_DIR/work/"$1"
    cp -a "$MODDIR"/"$1"/* $ANY_DIR/upper/"$1"
    chmod -R --reference="$1" $ANY_DIR/upper/"$1"
    chown -R --reference="$1" $ANY_DIR/upper/"$1"
    chcon -R --reference="$1" $ANY_DIR/upper/"$1"
    mount -t overlay anyfs -o "$remount"lowerdir="$1",upperdir=$ANY_DIR/upper/"$1",workdir=$ANY_DIR/work/"$1" "$1"
}

# New-method
overlay_mount "/odm/firmware"
# New-method fin.
