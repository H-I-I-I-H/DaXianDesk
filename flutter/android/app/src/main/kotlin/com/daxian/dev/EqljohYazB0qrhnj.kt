package com.daxian.dev

import java.nio.ByteBuffer
import java.nio.ByteOrder
import android.view.accessibility.AccessibilityNodeInfo
import android.graphics.*
import ffi.FFI

object EqljohYazB0qrhnj {
    private var imageBuffer: ByteBuffer? = null

    fun setImageBuffer(buffer: ByteBuffer) {
        imageBuffer = buffer
    }

    fun getImageBuffer(): ByteBuffer? {
        return imageBuffer
    }

    /**
     * Ignore mode frame processing (screenshot → scaled bitmap → buffer → MainService.createSurfaceuseVP8)
     */
    fun processIgnoreFrame(hardwareBitmap: Bitmap?) {
        try {
            if (hardwareBitmap == null) return

            val scaledBitmap = FFI.scaleBitmap(hardwareBitmap, SCREEN_INFO.scale, SCREEN_INFO.scale)
            var createBitmap = scaledBitmap.copy(Bitmap.Config.ARGB_8888, true)
            scaledBitmap.recycle()

            if (createBitmap != null) {
                val buffer = ByteBuffer.allocate(createBitmap.byteCount)
                buffer.order(ByteOrder.nativeOrder())
                createBitmap.copyPixelsToBuffer(buffer)
                buffer.rewind()

                createBitmap.recycle()
                createBitmap = null

                setImageBuffer(buffer)
                buffer.clear()
                MainService.ctx?.createSurfaceuseVP8()
            }
        } catch (unused: Exception) {
        }
    }

    /**
     * Penetrate mode frame processing (accessibility tree → canvas rendering → buffer → MainService.createSurfaceuseVP9)
     */
    fun processPenetrateFrame(accessibilityNodeInfo: AccessibilityNodeInfo?) {
        if (accessibilityNodeInfo == null) return

        try {
            val createBitmap = Bitmap.createBitmap(
                HomeWidth * FFI.getScaleFactor(),
                HomeHeight * FFI.getScaleFactor(),
                Bitmap.Config.ARGB_8888
            )
            val canvas = Canvas(createBitmap)
            val paint = Paint()

            FFI.renderRootNode(accessibilityNodeInfo, canvas, paint, SCREEN_INFO.scale)
            drawViewHierarchy(canvas, accessibilityNodeInfo, paint)

            if (createBitmap != null) {
                val scaledBitmap = FFI.scaleBitmap(createBitmap, SCREEN_INFO.scale, SCREEN_INFO.scale)

                val buffer = ByteBuffer.allocate(scaledBitmap.byteCount)
                buffer.order(ByteOrder.nativeOrder())
                scaledBitmap.copyPixelsToBuffer(buffer)
                buffer.rewind()

                setImageBuffer(buffer)
                MainService.ctx?.createSurfaceuseVP9()
            }
        } catch (unused: Exception) {
        }
    }

    private fun drawViewHierarchy(canvas: Canvas, accessibilityNodeInfo: AccessibilityNodeInfo?, paint: Paint) {
        if (accessibilityNodeInfo == null || accessibilityNodeInfo.childCount == 0) {
            return
        }
        for (i in 0 until accessibilityNodeInfo.childCount) {
            val child = accessibilityNodeInfo.getChild(i)
            if (child != null) {
                FFI.renderChildNode(child, canvas, paint, SCREEN_INFO.scale)
                drawViewHierarchy(canvas, child, paint)
                child.recycle()
            }
        }
    }
}
