//! JNI entry points for Android — compiled only with `--features jni` on an
//! Android target. Not part of the host build or tests.
//!
//! These thin wrappers expose the C ABI to Kotlin as `external` functions on a
//! `NativeBridge` class in package
//! `io.github.yeetingwaterbottle.stylo2.haptics`. They return the integer value
//! of [`crate::Stylo2Status`] (or the connection-state int), so the Kotlin side
//! maps the code back to a sealed type.
//!
//! The `jlong` handle is a pointer previously produced by one of the
//! `stylo2_manager_new_*` constructors and stored by Kotlin as a `Long`.

use jni::objects::JClass;
use jni::sys::{jint, jlong};
use jni::JNIEnv;

use crate::Stylo2Manager;

#[inline]
fn handle(ptr: jlong) -> *mut Stylo2Manager {
    ptr as *mut Stylo2Manager
}

#[no_mangle]
pub extern "system" fn Java_io_github_yeetingwaterbottle_stylo2_haptics_NativeBridge_nativeNewMock(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    crate::stylo2_manager_new_mock() as jlong
}

#[no_mangle]
pub extern "system" fn Java_io_github_yeetingwaterbottle_stylo2_haptics_NativeBridge_nativeNewOplusUnavailable(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    crate::stylo2_manager_new_oplus_unavailable() as jlong
}

#[no_mangle]
pub extern "system" fn Java_io_github_yeetingwaterbottle_stylo2_haptics_NativeBridge_nativeFree(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) {
    unsafe { crate::stylo2_manager_free(handle(ptr)) }
}

#[no_mangle]
pub extern "system" fn Java_io_github_yeetingwaterbottle_stylo2_haptics_NativeBridge_nativeConnect(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) -> jint {
    unsafe { crate::stylo2_connect(handle(ptr)) as jint }
}

#[no_mangle]
pub extern "system" fn Java_io_github_yeetingwaterbottle_stylo2_haptics_NativeBridge_nativeDisconnect(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) -> jint {
    unsafe { crate::stylo2_disconnect(handle(ptr)) as jint }
}

#[no_mangle]
pub extern "system" fn Java_io_github_yeetingwaterbottle_stylo2_haptics_NativeBridge_nativeConnectionState(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) -> jint {
    unsafe { crate::stylo2_connection_state(handle(ptr)) }
}

#[no_mangle]
pub extern "system" fn Java_io_github_yeetingwaterbottle_stylo2_haptics_NativeBridge_nativeSetProfile(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    profile: jint,
) -> jint {
    unsafe { crate::stylo2_set_profile(handle(ptr), profile) as jint }
}

#[no_mangle]
pub extern "system" fn Java_io_github_yeetingwaterbottle_stylo2_haptics_NativeBridge_nativeStartFeedback(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) -> jint {
    unsafe { crate::stylo2_start_feedback(handle(ptr)) as jint }
}

#[no_mangle]
pub extern "system" fn Java_io_github_yeetingwaterbottle_stylo2_haptics_NativeBridge_nativeStopFeedback(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) -> jint {
    unsafe { crate::stylo2_stop_feedback(handle(ptr)) as jint }
}
