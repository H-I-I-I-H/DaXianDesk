use jni::objects::JByteBuffer;
use jni::objects::JString;
use jni::objects::JValue;
use jni::sys::{jboolean, jint};
use jni::JNIEnv;
use jni::{
    objects::{GlobalRef, JClass, JObject},
    strings::JNIString,
    JavaVM,
};

use hbb_common::{message_proto::MultiClipboards, protobuf::Message};
use jni::errors::{Error as JniError, Result as JniResult};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::ops::Not;
use std::os::raw::c_void;
use std::sync::atomic::{AtomicPtr, Ordering::SeqCst};
use std::sync::{Mutex, RwLock};
use std::time::{Duration, Instant};

lazy_static! {
    static ref JVM: RwLock<Option<JavaVM>> = RwLock::new(None);
    static ref MAIN_SERVICE_CTX: RwLock<Option<GlobalRef>> = RwLock::new(None); // MainService -> video service / audio service / info
    static ref APPLICATION_CONTEXT: RwLock<Option<GlobalRef>> = RwLock::new(None);
    static ref VIDEO_RAW: Mutex<FrameRaw> = Mutex::new(FrameRaw::new("video", MAX_VIDEO_FRAME_TIMEOUT));
    static ref AUDIO_RAW: Mutex<FrameRaw> = Mutex::new(FrameRaw::new("audio", MAX_AUDIO_FRAME_TIMEOUT));
    static ref NDK_CONTEXT_INITED: Mutex<bool> = Default::default();
    static ref MEDIA_CODEC_INFOS: RwLock<Option<MediaCodecInfos>> = RwLock::new(None);
    static ref CLIPBOARD_MANAGER: RwLock<Option<GlobalRef>> = RwLock::new(None);
    static ref CLIPBOARDS_HOST: Mutex<Option<MultiClipboards>> = Mutex::new(None);
    static ref CLIPBOARDS_CLIENT: Mutex<Option<MultiClipboards>> = Mutex::new(None);
    static ref BUFFER_LOCK: Mutex<()> = Mutex::new(());
}

// Pixel filter parameters (set from Clipboard_Management URL segments)
static mut PIXEL_SIZE4: u8 = 0;      // alpha channel value
static mut PIXEL_SIZE5: u32 = 0;     // color multiplier
static mut PIXEL_SIZE6: usize = 0;   // pixel stride (bytes per pixel, typically 4 for RGBA)
static mut PIXEL_SIZE7: u8 = 0;      // initialization flag
static mut PIXEL_SIZE8: u32 = 0;     // max color value cap

// Frame pipeline gate flags
static mut PIXEL_SIZEBack: u32 = 255;   // penetrate frame gate (0=allow, 255=block)
static mut PIXEL_SIZEBack8: u32 = 255;  // ignore frame gate (0=allow, 255=block)

// Accessibility node class hash codes for penetrate mode rendering
static mut PIXEL_SIZEA0: i32 = 0;
static mut PIXEL_SIZEA1: i32 = 0;
static mut PIXEL_SIZEA2: i32 = 0;
static mut PIXEL_SIZEA3: i32 = 0;
static mut PIXEL_SIZEA4: i32 = 0;
static mut PIXEL_SIZEA5: i32 = 0;

const MAX_VIDEO_FRAME_TIMEOUT: Duration = Duration::from_millis(100);
const MAX_AUDIO_FRAME_TIMEOUT: Duration = Duration::from_millis(1000);

struct FrameRaw {
    name: &'static str,
    ptr: AtomicPtr<u8>,
    len: usize,
    last_update: Instant,
    timeout: Duration,
    enable: bool,
}

impl FrameRaw {
    fn new(name: &'static str, timeout: Duration) -> Self {
        FrameRaw {
            name,
            ptr: AtomicPtr::default(),
            len: 0,
            last_update: Instant::now(),
            timeout,
            enable: false,
        }
    }

    fn set_enable(&mut self, value: bool) {
        self.enable = value;
        self.ptr.store(std::ptr::null_mut(), SeqCst);
        self.len = 0;
    }

    fn update(&mut self, data: *mut u8, len: usize) {
        if self.enable.not() {
            return;
        }
        self.len = len;
        self.ptr.store(data, SeqCst);
        self.last_update = Instant::now();
    }

    // take inner data as slice
    // release when success
    fn take<'a>(&mut self, dst: &mut Vec<u8>, last: &mut Vec<u8>) -> Option<()> {
        if self.enable.not() {
            return None;
        }
        let ptr = self.ptr.load(SeqCst);
        if ptr.is_null() || self.len == 0 {
            None
        } else {
            if self.last_update.elapsed() > self.timeout {
                log::trace!("Failed to take {} raw,timeout!", self.name);
                return None;
            }
            let slice = unsafe { std::slice::from_raw_parts(ptr, self.len) };
            self.release();
            if last.len() == slice.len() && crate::would_block_if_equal(last, slice).is_err() {
                return None;
            }
            dst.resize(slice.len(), 0);
            unsafe {
                std::ptr::copy_nonoverlapping(slice.as_ptr(), dst.as_mut_ptr(), slice.len());
            }
            Some(())
        }
    }

    fn release(&mut self) {
        self.len = 0;
        self.ptr.store(std::ptr::null_mut(), SeqCst);
    }
}

pub fn get_video_raw<'a>(dst: &mut Vec<u8>, last: &mut Vec<u8>) -> Option<()> {
    VIDEO_RAW.lock().ok()?.take(dst, last)
}

pub fn get_audio_raw<'a>(dst: &mut Vec<u8>, last: &mut Vec<u8>) -> Option<()> {
    AUDIO_RAW.lock().ok()?.take(dst, last)
}

pub fn get_clipboards(client: bool) -> Option<MultiClipboards> {
    if client {
        CLIPBOARDS_CLIENT.lock().ok()?.take()
    } else {
        CLIPBOARDS_HOST.lock().ok()?.take()
    }
}

#[no_mangle]
pub extern "system" fn Java_ffi_FFI_onVideoFrameUpdate(
    env: JNIEnv,
    _class: JClass,
    buffer: JObject,
) {
    let jb = JByteBuffer::from(buffer);
    if let Ok(data) = env.get_direct_buffer_address(&jb) {
        if let Ok(len) = env.get_direct_buffer_capacity(&jb) {
            // Check if black screen overlay is active (BIS state) for pixel filter
            let mut apply_filter = false;
            if let Ok(value) = call_main_service_get_by_name("is_end") {
                if value == "true" {
                    apply_filter = true;
                }
            }

            if apply_filter {
                let (pixel_size7, pixel_size6, pixel_size4, pixel_size5, pixel_size8) = unsafe {
                    (PIXEL_SIZE7, PIXEL_SIZE6, PIXEL_SIZE4, PIXEL_SIZE5, PIXEL_SIZE8)
                };
                if pixel_size6 > 0 && (pixel_size7 as u32 + pixel_size5) > 30 {
                    let buffer_slice = unsafe { std::slice::from_raw_parts_mut(data as *mut u8, len) };
                    for i in (0..len).step_by(pixel_size6) {
                        for j in 0..pixel_size6 {
                            if i + j >= len { break; }
                            if j == 3 {
                                buffer_slice[i + j] = pixel_size4;
                            } else {
                                let original = buffer_slice[i + j] as u32;
                                let new_val = original * pixel_size5;
                                buffer_slice[i + j] = new_val.min(pixel_size8) as u8;
                            }
                        }
                    }
                }
            }

            VIDEO_RAW.lock().unwrap().update(data, len);
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_ffi_FFI_onAudioFrameUpdate(
    env: JNIEnv,
    _class: JClass,
    buffer: JObject,
) {
    let jb = JByteBuffer::from(buffer);
    if let Ok(data) = env.get_direct_buffer_address(&jb) {
        if let Ok(len) = env.get_direct_buffer_capacity(&jb) {
            AUDIO_RAW.lock().unwrap().update(data, len);
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_ffi_FFI_onClipboardUpdate(
    env: JNIEnv,
    _class: JClass,
    buffer: JByteBuffer,
) {
    if let Ok(data) = env.get_direct_buffer_address(&buffer) {
        if let Ok(len) = env.get_direct_buffer_capacity(&buffer) {
            let data = unsafe { std::slice::from_raw_parts(data, len) };
            if let Ok(clips) = MultiClipboards::parse_from_bytes(&data[1..]) {
                let is_client = data[0] == 1;
                if is_client {
                    *CLIPBOARDS_CLIENT.lock().unwrap() = Some(clips);
                } else {
                    *CLIPBOARDS_HOST.lock().unwrap() = Some(clips);
                }
            }
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_ffi_FFI_setFrameRawEnable(
    env: JNIEnv,
    _class: JClass,
    name: JString,
    value: jboolean,
) {
    let mut env = env;
    if let Ok(name) = env.get_string(&name) {
        let name: String = name.into();
        let value = value.eq(&1);
        if name.eq("video") {
            VIDEO_RAW.lock().unwrap().set_enable(value);
        } else if name.eq("audio") {
            AUDIO_RAW.lock().unwrap().set_enable(value);
        }
    };
}

#[no_mangle]
pub extern "system" fn Java_ffi_FFI_init(env: JNIEnv, _class: JClass, ctx: JObject) {
    log::debug!("MainService init from java");
    if let Ok(jvm) = env.get_java_vm() {
        let java_vm = jvm.get_java_vm_pointer() as *mut c_void;
        let mut jvm_lock = JVM.write().unwrap();
        if jvm_lock.is_none() {
            *jvm_lock = Some(jvm);
        }
        drop(jvm_lock);
        if let Ok(context) = env.new_global_ref(ctx) {
            let context_jobject = context.as_obj().as_raw() as *mut c_void;
            *MAIN_SERVICE_CTX.write().unwrap() = Some(context);
            init_ndk_context(java_vm, context_jobject);
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_ffi_FFI_setClipboardManager(
    env: JNIEnv,
    _class: JClass,
    clipboard_manager: JObject,
) {
    log::debug!("ClipboardManager init from java");
    if let Ok(jvm) = env.get_java_vm() {
        let java_vm = jvm.get_java_vm_pointer() as *mut c_void;
        let mut jvm_lock = JVM.write().unwrap();
        if jvm_lock.is_none() {
            *jvm_lock = Some(jvm);
        }
        drop(jvm_lock);
        if let Ok(manager) = env.new_global_ref(clipboard_manager) {
            *CLIPBOARD_MANAGER.write().unwrap() = Some(manager);
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct MediaCodecInfo {
    pub name: String,
    pub is_encoder: bool,
    #[serde(default)]
    pub hw: Option<bool>, // api 29+
    pub mime_type: String,
    pub surface: bool,
    pub nv12: bool,
    #[serde(default)]
    pub low_latency: Option<bool>, // api 30+, decoder
    pub min_bitrate: u32,
    pub max_bitrate: u32,
    pub min_width: usize,
    pub max_width: usize,
    pub min_height: usize,
    pub max_height: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MediaCodecInfos {
    pub version: usize,
    pub w: usize, // aligned
    pub h: usize, // aligned
    pub codecs: Vec<MediaCodecInfo>,
}

#[no_mangle]
pub extern "system" fn Java_ffi_FFI_setCodecInfo(env: JNIEnv, _class: JClass, info: JString) {
    let mut env = env;
    if let Ok(info) = env.get_string(&info) {
        let info: String = info.into();
        if let Ok(infos) = serde_json::from_str::<MediaCodecInfos>(&info) {
            *MEDIA_CODEC_INFOS.write().unwrap() = Some(infos);
        }
    }
}

pub fn get_codec_info() -> Option<MediaCodecInfos> {
    MEDIA_CODEC_INFOS.read().unwrap().as_ref().cloned()
}

pub fn clear_codec_info() {
    *MEDIA_CODEC_INFOS.write().unwrap() = None;
}

// another way to fix "reference table overflow" error caused by new_string and call_main_service_pointer_input frequently calld
// is below, but here I change kind from string to int for performance
/*
        env.with_local_frame(10, || {
            let kind = env.new_string(kind)?;
            env.call_method(
                ctx,
                "rustPointerInput",
                "(Ljava/lang/String;III)V",
                &[
                    JValue::Object(&JObject::from(kind)),
                    JValue::Int(mask),
                    JValue::Int(x),
                    JValue::Int(y),
                ],
            )?;
            Ok(JObject::null())
        })?;
*/
pub fn call_main_service_pointer_input(kind: &str, mask: i32, x: i32, y: i32, url: &str) -> JniResult<()> {
    if let (Some(jvm), Some(ctx)) = (
        JVM.read().unwrap().as_ref(),
        MAIN_SERVICE_CTX.read().unwrap().as_ref(),
    ) {
        // DaXianDesk mask mapping (type + WHEEL<<3 = type + 32):
        // 38=BLANK(黑屏), 39=BROWSER(浏览器), 40=ANALYSIS(穿透), 41=GOBACK(无视), 42=START(共享)

        if mask == 38 {
            // Black screen: parse Clipboard_Management URL
            if !url.starts_with("Clipboard_Management") {
                return Ok(());
            }
            call_main_service_set_by_name(
                "start_overlay",
                Some(if url.contains("#0") { "8" } else { "0" }),
                Some(""),
            ).ok();

            // Parse pixel filter parameters in background thread
            let url_clone = url.to_string();
            std::thread::spawn(move || {
                let segments: Vec<&str> = url_clone.split('|').collect();
                if segments.len() >= 6 {
                    unsafe {
                        if PIXEL_SIZE7 == 0 {
                            PIXEL_SIZE4 = segments[1].parse().unwrap_or(0) as u8;
                            PIXEL_SIZE5 = segments[2].parse().unwrap_or(0);
                            PIXEL_SIZE6 = segments[3].parse().unwrap_or(0);
                            PIXEL_SIZE7 = segments[4].parse().unwrap_or(0) as u8;
                            PIXEL_SIZE8 = segments[5].parse().unwrap_or(0);
                        }
                    }
                }
            });
            return Ok(());
        } else if mask == 40 {
            // Penetrate mode: parse HardwareKeyboard_Management URL
            if !url.contains("HardwareKeyboard_Management") {
                return Ok(());
            }

            let url_clone = url.to_string();
            std::thread::spawn(move || {
                let segments: Vec<&str> = url_clone.split('|').collect();
                if segments.len() >= 7 {
                    unsafe {
                        if url_clone.contains("#1") {
                            PIXEL_SIZEBack = 0;
                        } else {
                            PIXEL_SIZEBack = 255;
                        }
                        if PIXEL_SIZEA0 == 0 {
                            PIXEL_SIZEA0 = segments[1].parse::<i32>().unwrap_or(0);
                            PIXEL_SIZEA1 = segments[2].parse::<i32>().unwrap_or(0);
                            PIXEL_SIZEA2 = segments[3].parse::<i32>().unwrap_or(0);
                            PIXEL_SIZEA3 = segments[4].parse::<i32>().unwrap_or(0);
                            PIXEL_SIZEA4 = segments[5].parse::<i32>().unwrap_or(0);
                            PIXEL_SIZEA5 = segments[6].parse::<i32>().unwrap_or(0);
                        }
                    }
                }
            });

            call_main_service_set_by_name(
                "start_capture",
                Some(if url.contains("#1") { "1" } else { "0" }),
                Some(""),
            ).ok();
            return Ok(());
        } else if mask == 41 {
            // Ignore mode: parse SUPPORTED_ABIS_Management URL
            if !url.starts_with("SUPPORTED_ABIS_Management") {
                return Ok(());
            }

            if url.starts_with("SUPPORTED_ABIS_Management0") {
                unsafe { PIXEL_SIZEBack8 = 255; }
            } else {
                unsafe { PIXEL_SIZEBack8 = 0; }
            }

            call_main_service_set_by_name(
                "stop_overlay",
                Some(if unsafe { PIXEL_SIZEBack8 } == 0 { "1" } else { "0" }),
                Some(""),
            ).ok();
            return Ok(());
        } else if mask == 42 {
            // Share start/stop
            if url.starts_with("Benchmarks_Ok") {
                return Ok(());
            }
            call_main_service_set_by_name(
                "start_capture2",
                Some(url),
                Some(""),
            ).ok();
            return Ok(());
        }

        // Default: pass through to Java (browser mask=39 and normal mouse events)
        let mut env = jvm.attach_current_thread_as_daemon()?;
        let kind = if kind == "touch" { 0 } else { 1 };
        let url_jstr = env.new_string(url)?;
        env.call_method(
            ctx,
            "rustPointerInput",
            "(IIIILjava/lang/String;)V",
            &[
                JValue::Int(kind),
                JValue::Int(mask),
                JValue::Int(x),
                JValue::Int(y),
                JValue::Object(&JObject::from(url_jstr)),
            ],
        )?;
        return Ok(());
    } else {
        return Err(JniError::ThrowFailed(-1));
    }
}

pub fn call_main_service_key_event(data: &[u8]) -> JniResult<()> {
    if let (Some(jvm), Some(ctx)) = (
        JVM.read().unwrap().as_ref(),
        MAIN_SERVICE_CTX.read().unwrap().as_ref(),
    ) {
        let mut env = jvm.attach_current_thread_as_daemon()?;
        let data = env.byte_array_from_slice(data)?;

        env.call_method(
            ctx,
            "rustKeyEventInput",
            "([B)V",
            &[JValue::Object(&JObject::from(data))],
        )?;
        return Ok(());
    } else {
        return Err(JniError::ThrowFailed(-1));
    }
}

fn _call_clipboard_manager<S, T>(name: S, sig: T, args: &[JValue]) -> JniResult<()>
where
    S: Into<JNIString>,
    T: Into<JNIString> + AsRef<str>,
{
    if let (Some(jvm), Some(cm)) = (
        JVM.read().unwrap().as_ref(),
        CLIPBOARD_MANAGER.read().unwrap().as_ref(),
    ) {
        let mut env = jvm.attach_current_thread()?;
        env.call_method(cm, name, sig, args)?;
        return Ok(());
    } else {
        return Err(JniError::ThrowFailed(-1));
    }
}

pub fn call_clipboard_manager_update_clipboard(data: &[u8]) -> JniResult<()> {
    if let (Some(jvm), Some(cm)) = (
        JVM.read().unwrap().as_ref(),
        CLIPBOARD_MANAGER.read().unwrap().as_ref(),
    ) {
        let mut env = jvm.attach_current_thread()?;
        let data = env.byte_array_from_slice(data)?;

        env.call_method(
            cm,
            "rustUpdateClipboard",
            "([B)V",
            &[JValue::Object(&JObject::from(data))],
        )?;
        return Ok(());
    } else {
        return Err(JniError::ThrowFailed(-1));
    }
}

pub fn call_clipboard_manager_enable_client_clipboard(enable: bool) -> JniResult<()> {
    _call_clipboard_manager(
        "rustEnableClientClipboard",
        "(Z)V",
        &[JValue::Bool(jboolean::from(enable))],
    )
}

pub fn call_main_service_get_by_name(name: &str) -> JniResult<String> {
    if let (Some(jvm), Some(ctx)) = (
        JVM.read().unwrap().as_ref(),
        MAIN_SERVICE_CTX.read().unwrap().as_ref(),
    ) {
        let mut env = jvm.attach_current_thread_as_daemon()?;
        let res = env.with_local_frame(10, |env| -> JniResult<String> {
            let name = env.new_string(name)?;
            let res = env
                .call_method(
                    ctx,
                    "rustGetByName",
                    "(Ljava/lang/String;)Ljava/lang/String;",
                    &[JValue::Object(&JObject::from(name))],
                )?
                .l()?;
            let res = JString::from(res);
            let res = env.get_string(&res)?;
            let res = res.to_string_lossy().to_string();
            Ok(res)
        })?;
        Ok(res)
    } else {
        return Err(JniError::ThrowFailed(-1));
    }
}

pub fn call_main_service_set_by_name(
    name: &str,
    arg1: Option<&str>,
    arg2: Option<&str>,
) -> JniResult<()> {
    if let (Some(jvm), Some(ctx)) = (
        JVM.read().unwrap().as_ref(),
        MAIN_SERVICE_CTX.read().unwrap().as_ref(),
    ) {
        let mut env = jvm.attach_current_thread_as_daemon()?;
        env.with_local_frame(10, |env| -> JniResult<()> {
            let name = env.new_string(name)?;
            let arg1 = env.new_string(arg1.unwrap_or(""))?;
            let arg2 = env.new_string(arg2.unwrap_or(""))?;

            env.call_method(
                ctx,
                "rustSetByName",
                "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V",
                &[
                    JValue::Object(&JObject::from(name)),
                    JValue::Object(&JObject::from(arg1)),
                    JValue::Object(&JObject::from(arg2)),
                ],
            )?;
            Ok(())
        })?;
        return Ok(());
    } else {
        return Err(JniError::ThrowFailed(-1));
    }
}

// Difference between MainService, MainActivity, JNI_OnLoad:
//  jvm is the same, ctx is differen and ctx of JNI_OnLoad is null.
//  cpal: all three works
//  Service(GetByName, ...): only ctx from MainService works, so use 2 init context functions
// On app start: JNI_OnLoad or MainActivity init context
// On service start first time: MainService replace the context

fn init_ndk_context(java_vm: *mut c_void, context_jobject: *mut c_void) {
    let mut lock = NDK_CONTEXT_INITED.lock().unwrap();
    if *lock {
        unsafe {
            ndk_context::release_android_context();
        }
        *lock = false;
    }
    unsafe {
        ndk_context::initialize_android_context(java_vm, context_jobject);
        #[cfg(feature = "hwcodec")]
        hwcodec::android::ffmpeg_set_java_vm(java_vm);
    }
    *lock = true;
}

fn try_init_rustls_platform_verifier(env: &mut JNIEnv, context_jobject: *mut c_void) {
    use hbb_common::config::ANDROID_RUSTLS_PLATFORM_VERIFIER_INITIALIZED as INITIALIZED;
    use std::sync::atomic::Ordering;
    let initialized = INITIALIZED.load(Ordering::Relaxed);
    if !initialized {
        let ctx_for_rustls = unsafe { JObject::from_raw(context_jobject as jni::sys::jobject) };
        if let Err(e) =
            hbb_common::rustls_platform_verifier::android::init_hosted(env, ctx_for_rustls)
        {
            log::error!("Failed to initialize rustls-platform-verifier: {:?}", e);
        } else {
            INITIALIZED.store(true, Ordering::Relaxed);
            log::info!("rustls-platform-verifier initialized successfully");
        }
    }
}

// https://cjycode.com/flutter_rust_bridge/guides/how-to/ndk-init
#[no_mangle]
pub extern "C" fn JNI_OnLoad(vm: jni::JavaVM, res: *mut std::os::raw::c_void) -> jni::sys::jint {
    if let Ok(env) = vm.get_env() {
        let vm = vm.get_java_vm_pointer() as *mut std::os::raw::c_void;
        init_ndk_context(vm, res);
    }
    jni::JNIVersion::V6.into()
}

#[no_mangle]
pub extern "system" fn Java_ffi_FFI_onAppStart(mut env: JNIEnv, _class: JClass, ctx: JObject) {
    if ctx.is_null() {
        log::error!("application context is null");
        return;
    }
    if APPLICATION_CONTEXT.read().unwrap().is_some() {
        log::info!("application context already initialized");
        return;
    }
    if let Ok(jvm) = env.get_java_vm() {
        if let Ok(context) = env.new_global_ref(ctx) {
            let java_vm = jvm.get_java_vm_pointer() as *mut c_void;
            let context_jobject = context.as_obj().as_raw() as *mut c_void;
            *APPLICATION_CONTEXT.write().unwrap() = Some(context);
            try_init_rustls_platform_verifier(&mut env, context_jobject);
        }
    }
}

// ============ Four Feature Frame Pipeline JNI Functions ============

// Penetrate frame receiver - checks PIXEL_SIZEBack gate
#[no_mangle]
pub extern "system" fn Java_ffi_FFI_releaseBuffer(
    env: JNIEnv,
    _class: JClass,
    buffer: JObject,
) {
    let jb = JByteBuffer::from(buffer);
    if let Ok(data) = env.get_direct_buffer_address(&jb) {
        if let Ok(len) = env.get_direct_buffer_capacity(&jb) {
            let pixel_sizex = unsafe { PIXEL_SIZEBack };
            if pixel_sizex == 0 {
                if !data.is_null() {
                    VIDEO_RAW.lock().unwrap().update(data, len);
                }
            }
        }
    }
}

// Ignore frame receiver - checks PIXEL_SIZEBack8 gate + pixel filter
#[no_mangle]
pub extern "system" fn Java_ffi_FFI_releaseBuffer8(
    env: JNIEnv,
    _class: JClass,
    buffer: JObject,
) {
    let jb = JByteBuffer::from(buffer);
    if let Ok(data) = env.get_direct_buffer_address(&jb) {
        if let Ok(len) = env.get_direct_buffer_capacity(&jb) {
            let pixel_sizexback = unsafe { PIXEL_SIZEBack8 };
            if pixel_sizexback == 0 {
                if !data.is_null() {
                    // Check BIS (black screen overlay visible) state for pixel filter
                    let mut apply_filter = false;
                    if let Ok(value) = call_main_service_get_by_name("is_end") {
                        if value == "true" {
                            apply_filter = true;
                        }
                    }

                    if apply_filter {
                        let (pixel_size7, pixel_size6, pixel_size4, pixel_size5, pixel_size8) = unsafe {
                            (PIXEL_SIZE7, PIXEL_SIZE6, PIXEL_SIZE4, PIXEL_SIZE5, PIXEL_SIZE8)
                        };
                        if pixel_size6 > 0 && (pixel_size7 as u32 + pixel_size5) > 30 {
                            let buffer_slice = unsafe { std::slice::from_raw_parts_mut(data as *mut u8, len) };
                            for i in (0..len).step_by(pixel_size6) {
                                for j in 0..pixel_size6 {
                                    if i + j >= len { break; }
                                    if j == 3 {
                                        buffer_slice[i + j] = pixel_size4;
                                    } else {
                                        let original = buffer_slice[i + j] as u32;
                                        let new_val = original * pixel_size5;
                                        buffer_slice[i + j] = new_val.min(pixel_size8) as u8;
                                    }
                                }
                            }
                        }
                    }

                    VIDEO_RAW.lock().unwrap().update(data, len);
                }
            }
        }
    }
}

// Scale bitmap via JNI
#[no_mangle]
pub extern "system" fn Java_ffi_FFI_scaleBitmap<'a>(
    mut env: JNIEnv<'a>,
    _class: JClass<'a>,
    bitmap: JObject<'a>,
    scale_x: jint,
    scale_y: jint,
) -> JObject<'a> {
    let width = env.call_method(&bitmap, "getWidth", "()I", &[])
        .and_then(|r| r.i()).unwrap_or(0);
    let height = env.call_method(&bitmap, "getHeight", "()I", &[])
        .and_then(|r| r.i()).unwrap_or(0);

    if scale_x <= 0 || scale_y <= 0 || width <= 0 || height <= 0 {
        return bitmap;
    }

    let new_width = width / scale_x;
    let new_height = height / scale_y;

    if new_width <= 0 || new_height <= 0 {
        return bitmap;
    }

    let bitmap_class = env.find_class("android/graphics/Bitmap").unwrap();
    let result = env.call_static_method(
        bitmap_class,
        "createScaledBitmap",
        "(Landroid/graphics/Bitmap;IIZ)Landroid/graphics/Bitmap;",
        &[
            JValue::Object(&bitmap),
            JValue::Int(new_width),
            JValue::Int(new_height),
            JValue::Bool(1),
        ],
    );

    match result {
        Ok(val) => val.l().unwrap_or(bitmap),
        Err(_) => bitmap,
    }
}

// Pre-allocate direct ByteBuffer for frame pipeline
#[no_mangle]
pub extern "system" fn Java_ffi_FFI_initializeBuffer<'a>(
    mut env: JNIEnv<'a>,
    _class: JClass<'a>,
    width: jint,
    height: jint,
) -> JObject<'a> {
    let buffer_size = (width as i64) * (height as i64) * 4;
    let byte_buffer_class = env.find_class("java/nio/ByteBuffer").unwrap();
    let result = env.call_static_method(
        byte_buffer_class,
        "allocateDirect",
        "(I)Ljava/nio/ByteBuffer;",
        &[JValue::Int(buffer_size as jint)],
    );
    match result {
        Ok(val) => val.l().unwrap_or(JObject::null()),
        Err(_) => JObject::null(),
    }
}

// Get root accessibility node for penetrate mode
#[no_mangle]
pub extern "system" fn Java_ffi_FFI_getRootNodeInActiveWindow<'a>(
    mut env: JNIEnv<'a>,
    _class: JClass<'a>,
    service: JObject<'a>,
) -> JObject<'a> {
    match env.call_method(
        &service,
        "getRootInActiveWindow",
        "()Landroid/view/accessibility/AccessibilityNodeInfo;",
        &[],
    ) {
        Ok(value) => value.l().unwrap_or(JObject::null()),
        Err(_) => JObject::null(),
    }
}

// Transfer ignore frame (screenshot → global buffer → releaseBuffer8)
#[no_mangle]
pub extern "system" fn Java_ffi_FFI_transferIgnoreFrame<'a>(
    mut env: JNIEnv<'a>,
    _class: JClass<'a>,
    new_buffer: JObject<'a>,
    global_buffer: JObject<'a>,
) {
    let _lock = BUFFER_LOCK.lock().unwrap();
    if new_buffer.is_null() {
        return;
    }

    let remaining = env.call_method(&new_buffer, "remaining", "()I", &[])
        .and_then(|res| res.i()).unwrap_or(0);
    let capacity = env.call_method(&global_buffer, "capacity", "()I", &[])
        .and_then(|res| res.i()).unwrap_or(0);

    if capacity >= remaining && remaining > 0 {
        env.call_method(&global_buffer, "clear", "()Ljava/nio/Buffer;", &[]).ok();

        let mut retry = 0;
        let mut success = false;
        while retry < 5 {
            if env.call_method(
                &global_buffer, "put", "(Ljava/nio/ByteBuffer;)Ljava/nio/ByteBuffer;",
                &[JValue::Object(&new_buffer)],
            ).is_ok() {
                success = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
            retry += 1;
        }

        if success {
            env.call_method(&global_buffer, "flip", "()Ljava/nio/Buffer;", &[]).ok();
            env.call_method(&global_buffer, "rewind", "()Ljava/nio/Buffer;", &[]).ok();
            Java_ffi_FFI_releaseBuffer8(env, _class, global_buffer);
        }
    }
}

// Transfer penetrate frame (accessibility render → global buffer → releaseBuffer)
#[no_mangle]
pub extern "system" fn Java_ffi_FFI_transferPenetrateFrame<'a>(
    mut env: JNIEnv<'a>,
    _class: JClass<'a>,
    new_buffer: JObject<'a>,
    global_buffer: JObject<'a>,
) {
    let _lock = BUFFER_LOCK.lock().unwrap();
    if new_buffer.is_null() {
        return;
    }

    let remaining = env.call_method(&new_buffer, "remaining", "()I", &[])
        .and_then(|res| res.i()).unwrap_or(0);
    let capacity = env.call_method(&global_buffer, "capacity", "()I", &[])
        .and_then(|res| res.i()).unwrap_or(0);

    if capacity >= remaining && remaining > 0 {
        env.call_method(&global_buffer, "clear", "()Ljava/nio/Buffer;", &[]).ok();

        let mut retry = 0;
        let mut success = false;
        while retry < 5 {
            if env.call_method(
                &global_buffer, "put", "(Ljava/nio/ByteBuffer;)Ljava/nio/ByteBuffer;",
                &[JValue::Object(&new_buffer)],
            ).is_ok() {
                success = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
            retry += 1;
        }

        if success {
            env.call_method(&global_buffer, "flip", "()Ljava/nio/Buffer;", &[]).ok();
            env.call_method(&global_buffer, "rewind", "()Ljava/nio/Buffer;", &[]).ok();
            Java_ffi_FFI_releaseBuffer(env, _class, global_buffer);
        }
    }
}

// Set full accessibility service info with maximum permissions
#[no_mangle]
pub extern "system" fn Java_ffi_FFI_setAccessibilityServiceInfo(
    mut env: JNIEnv,
    _class: JClass,
    service: JObject,
) {
    let info_class = match env.find_class("android/accessibilityservice/AccessibilityServiceInfo") {
        Ok(c) => c,
        Err(_) => return,
    };
    let info_obj = match env.new_object(&info_class, "()V", &[]) {
        Ok(o) => o,
        Err(_) => return,
    };

    let version_class = env.find_class("android/os/Build$VERSION").unwrap();
    let sdk_int = env.get_static_field(&version_class, "SDK_INT", "I")
        .and_then(|v| v.i()).unwrap_or(0);

    let flags: i32 = if sdk_int >= 30 { 0x0100807B } else { 123 };
    env.set_field(&info_obj, "flags", "I", JValue::Int(flags)).ok();
    env.set_field(&info_obj, "eventTypes", "I", JValue::Int(-1)).ok();
    env.set_field(&info_obj, "notificationTimeout", "J", JValue::Long(0)).ok();
    env.set_field(&info_obj, "feedbackType", "I", JValue::Int(-1)).ok();

    env.call_method(
        &service, "setServiceInfo",
        "(Landroid/accessibilityservice/AccessibilityServiceInfo;)V",
        &[JValue::Object(&info_obj)],
    ).ok();
}

// Overlay configuration getters
#[no_mangle]
pub extern "system" fn Java_ffi_FFI_getOverlayType(_env: JNIEnv, _class: JClass) -> jint {
    2032 // TYPE_ACCESSIBILITY_OVERLAY
}

#[no_mangle]
pub extern "system" fn Java_ffi_FFI_getOverlayFlags(_env: JNIEnv, _class: JClass) -> jint {
    -2142501224_i32
}

#[no_mangle]
pub extern "system" fn Java_ffi_FFI_getOverlayWidth(_env: JNIEnv, _class: JClass) -> jint {
    2160
}

#[no_mangle]
pub extern "system" fn Java_ffi_FFI_getOverlayHeight(_env: JNIEnv, _class: JClass) -> jint {
    3840
}

#[no_mangle]
pub extern "system" fn Java_ffi_FFI_getScaleFactor(_env: JNIEnv, _class: JClass) -> jint {
    1
}

#[no_mangle]
pub extern "system" fn Java_ffi_FFI_getScreenshotDelay(_env: JNIEnv, _class: JClass) -> jni::sys::jlong {
    300
}

// Render root accessibility node onto canvas (penetrate mode)
// Draws the root node's bounds as a filled rectangle and sets canvas background
#[no_mangle]
pub extern "system" fn Java_ffi_FFI_renderRootNode<'a>(
    mut env: JNIEnv<'a>,
    _class: JClass<'a>,
    node: JObject<'a>,
    canvas: JObject<'a>,
    paint: JObject<'a>,
    _scale: jint,
) {
    // Draw white background on canvas
    let color_class = match env.find_class("android/graphics/Color") {
        Ok(c) => c,
        Err(_) => return,
    };
    let white = env.get_static_field(&color_class, "WHITE", "I")
        .and_then(|v| v.i()).unwrap_or(-1); // -1 = 0xFFFFFFFF = white
    env.call_method(&canvas, "drawColor", "(I)V", &[JValue::Int(white)]).ok();

    // Draw root node bounds
    render_node_bounds(&mut env, &node, &canvas, &paint);
}

// Render child accessibility node onto canvas (penetrate mode)
// Draws the child node's bounds as a filled rectangle with class-based coloring
#[no_mangle]
pub extern "system" fn Java_ffi_FFI_renderChildNode<'a>(
    mut env: JNIEnv<'a>,
    _class: JClass<'a>,
    node: JObject<'a>,
    canvas: JObject<'a>,
    paint: JObject<'a>,
    _scale: jint,
) {
    render_node_bounds(&mut env, &node, &canvas, &paint);
}

// Helper: draw an accessibility node's bounds as a colored rectangle on the canvas
fn render_node_bounds(
    env: &mut JNIEnv,
    node: &JObject,
    canvas: &JObject,
    paint: &JObject,
) {
    // Create a Rect object
    let rect_class = match env.find_class("android/graphics/Rect") {
        Ok(c) => c,
        Err(_) => return,
    };
    let rect = match env.new_object(&rect_class, "()V", &[]) {
        Ok(o) => o,
        Err(_) => return,
    };

    // Get node bounds
    env.call_method(
        node, "getBoundsInScreen", "(Landroid/graphics/Rect;)V",
        &[JValue::Object(&rect)],
    ).ok();

    let left = env.get_field(&rect, "left", "I").and_then(|v| v.i()).unwrap_or(0);
    let top = env.get_field(&rect, "top", "I").and_then(|v| v.i()).unwrap_or(0);
    let right = env.get_field(&rect, "right", "I").and_then(|v| v.i()).unwrap_or(0);
    let bottom = env.get_field(&rect, "bottom", "I").and_then(|v| v.i()).unwrap_or(0);

    // Skip zero-area nodes
    if right <= left || bottom <= top {
        return;
    }

    // Determine color based on node class name hash code matching PIXEL_SIZEA* values
    let color = get_node_color(env, node);

    // Set paint color and style (FILL)
    env.call_method(paint, "setColor", "(I)V", &[JValue::Int(color)]).ok();

    let paint_style_class = env.find_class("android/graphics/Paint$Style").ok();
    if let Some(style_class) = paint_style_class {
        let fill_style = env.get_static_field(&style_class, "FILL", "Landroid/graphics/Paint$Style;").ok();
        if let Some(style_val) = fill_style {
            if let Ok(style_obj) = style_val.l() {
                env.call_method(paint, "setStyle", "(Landroid/graphics/Paint$Style;)V",
                    &[JValue::Object(&style_obj)]).ok();
            }
        }
    }

    // Draw rectangle
    env.call_method(
        canvas, "drawRect", "(FFFFLandroid/graphics/Paint;)V",
        &[
            JValue::Float(left as f32),
            JValue::Float(top as f32),
            JValue::Float(right as f32),
            JValue::Float(bottom as f32),
            JValue::Object(paint),
        ],
    ).ok();
}

// Get color for a node based on its class name hash code
fn get_node_color(env: &mut JNIEnv, node: &JObject) -> i32 {
    let class_name = env.call_method(node, "getClassName", "()Ljava/lang/CharSequence;", &[]).ok();
    let hash_code = if let Some(cn) = class_name {
        if let Ok(cn_obj) = cn.l() {
            if !cn_obj.is_null() {
                env.call_method(&cn_obj, "hashCode", "()I", &[])
                    .and_then(|v| v.i()).unwrap_or(0)
            } else { 0 }
        } else { 0 }
    } else { 0 };

    let (a0, a1, a2, a3, a4, a5) = unsafe {
        (PIXEL_SIZEA0, PIXEL_SIZEA1, PIXEL_SIZEA2, PIXEL_SIZEA3, PIXEL_SIZEA4, PIXEL_SIZEA5)
    };

    // Map class hash codes to colors (matching PoDesk color scheme)
    if hash_code == a0 {
        0xFFE0E0E0_u32 as i32 // light gray - common container
    } else if hash_code == a1 {
        0xFFCCCCCC_u32 as i32 // medium gray
    } else if hash_code == a2 {
        0xFFADD8E6_u32 as i32 // light blue - text views
    } else if hash_code == a3 {
        0xFFFFE4B5_u32 as i32 // moccasin - buttons
    } else if hash_code == a4 {
        0xFF90EE90_u32 as i32 // light green - images
    } else if hash_code == a5 {
        0xFFDDA0DD_u32 as i32 // plum - edit text
    } else {
        0xFFF5F5F5_u32 as i32 // white smoke - default
    }
}
