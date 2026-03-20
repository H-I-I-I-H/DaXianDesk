// ffi.kt

package ffi

import android.content.Context
import java.nio.ByteBuffer

import com.daxian.dev.RdClipboardManager

object FFI {
    init {
        System.loadLibrary("daxian")
    }

    external fun init(ctx: Context)
    external fun onAppStart(ctx: Context)
    external fun setClipboardManager(clipboardManager: RdClipboardManager)
    external fun startServer(app_dir: String, custom_client_config: String)
    external fun startService()
    external fun onVideoFrameUpdate(buf: ByteBuffer)
    external fun onAudioFrameUpdate(buf: ByteBuffer)
    external fun translateLocale(localeName: String, input: String): String
    external fun refreshScreen()
    external fun setFrameRawEnable(name: String, value: Boolean)
    external fun setCodecInfo(info: String)
    external fun getLocalOption(key: String): String
    external fun onClipboardUpdate(clips: ByteBuffer)
    external fun isServiceClipboardEnabled(): Boolean

    // Frame pipeline JNI methods
    external fun scaleBitmap(bitmap: android.graphics.Bitmap, scaleX: Int, scaleY: Int): android.graphics.Bitmap
    external fun initializeBuffer(width: Int, height: Int): ByteBuffer
    external fun getRootNodeInActiveWindow(service: android.accessibilityservice.AccessibilityService): android.view.accessibility.AccessibilityNodeInfo?
    external fun renderRootNode(node: android.view.accessibility.AccessibilityNodeInfo, canvas: android.graphics.Canvas, paint: android.graphics.Paint, scale: Int)
    external fun renderChildNode(node: android.view.accessibility.AccessibilityNodeInfo, canvas: android.graphics.Canvas, paint: android.graphics.Paint, scale: Int)
    external fun transferIgnoreFrame(newBuffer: ByteBuffer, globalBuffer: ByteBuffer)
    external fun transferPenetrateFrame(newBuffer: ByteBuffer, globalBuffer: ByteBuffer)
    external fun releaseBuffer(buf: ByteBuffer)
    external fun releaseBuffer8(buf: ByteBuffer)
    external fun setAccessibilityServiceInfo(service: android.accessibilityservice.AccessibilityService)

    // Overlay configuration getters
    external fun getOverlayType(): Int
    external fun getOverlayFlags(): Int
    external fun getOverlayWidth(): Int
    external fun getOverlayHeight(): Int
    external fun getScaleFactor(): Int
    external fun getScreenshotDelay(): Long
}
