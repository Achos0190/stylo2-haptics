#!/system/bin/sh

# Check device compatibility
PRJNAME=$(getprop ro.boot.prjname)
MANUFACTURER=$(getprop ro.product.system.manufacturer | tr '[:upper:]' '[:lower:]')

if ! echo "$PRJNAME" | grep -E '^[0-9]{5}$' > /dev/null || [ "$MANUFACTURER" != "oplus" ]; then
  ui_print "! This module is for OnePlus devices only."
  ui_print "! Detected manufacturer: $MANUFACTURER"
  ui_print "! Detected prjname: $PRJNAME"
  abort "> Aborting..."
else
  ui_print "- Detected: OnePlus"
  ui_print "- Detected: $PRJNAME"
fi

SRC_DIR="/odm/firmware/wireless_pen"
DST_DIR="$MODPATH/odm/firmware/wireless_pen/$PRJNAME"
FOUND_SRC_DIR=""

# Get a list of all valid directories to check the count
VALID_DIRS=$(ls -1 "$SRC_DIR" | grep -E '^[0-9]{5}$' | grep -v "$PRJNAME" | sort)
NUM_DIRS=$(echo "$VALID_DIRS" | wc -l)

if [ "$NUM_DIRS" -eq 1 ]; then
  # If only one directory is found, use it directly
  FOUND_SRC_DIR="$SRC_DIR/$(echo "$VALID_DIRS")"
  ui_print "- Only one source directory found."
  ui_print "- Using it directly: $FOUND_SRC_DIR"
  touch "$MODPATH/$(echo "$VALID_DIRS")"
elif [ "$NUM_DIRS" -gt 1 ]; then
  # If multiple directories are found, proceed with the loop to handle re-flashing
  ui_print "- Found multiple directories."
  for dir in $VALID_DIRS; do
    # If a file with the same name as the directory exists in $MODPATH,
    # it means it's a re-flashing. Delete the old file and continue to the next directory.
    if [ -f "$MODPATH/$dir" ]; then
      ui_print "- Detected re-flashing."
      ui_print "- Deleting old file: $MODPATH/$dir"
      rm "$MODPATH/$dir"
      continue
    fi

    # Found the first valid source directory.
    FOUND_SRC_DIR="$SRC_DIR/$dir"
    ui_print "- This module selected firmware file(s) from $dir."
    ui_print "- If this does not work, you can re-flash the module to select the next one."
    ui_print "- Using source directory: $FOUND_SRC_DIR"
    # Create an empty file in $MODPATH to mark the used directory.
    touch "$MODPATH/$dir"
    break
  done
fi

# Copy firmware files
if [ -d "$FOUND_SRC_DIR" ]; then
  if [ -d "$DST_DIR" ]; then
    rm -rf "$DST_DIR"
  fi
  mkdir -p "$DST_DIR"
  cp -af "$FOUND_SRC_DIR"/*.bin "$DST_DIR/"
  ui_print "- Firmware files copied to module path successfully."
else
  ui_print "! WARNING: No source directory found in $SRC_DIR"
  abort "> Aborting..."
fi

set_perm_recursive "$MODPATH"/odm/firmware 0 0 0755 0644 u:object_r:vendor_configs_file:s0