package io.github.yeetingwaterbottle.stylo2.bleinspector

import io.github.yeetingwaterbottle.stylo2.bleinspector.ble.HexCodec
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class HexCodecTest {
    @Test
    fun parsesSpacedHex() {
        assertArrayEquals(byteArrayOf(0x01, 0x0A, 0xFF.toByte()), HexCodec.parseHex("01 0A FF"))
    }

    @Test
    fun parsesCompactPrefixedHex() {
        assertArrayEquals(byteArrayOf(0x0A, 0x1B), HexCodec.parseHex("0x0a1b"))
    }

    @Test
    fun rejectsOddLength() {
        assertNull(HexCodec.parseHex("0A1"))
    }

    @Test
    fun rejectsNonHex() {
        assertNull(HexCodec.parseHex("zz"))
    }

    @Test
    fun rejectsEmpty() {
        assertNull(HexCodec.parseHex("   "))
    }

    @Test
    fun rendersHexUppercaseSpaced() {
        assertEquals("01 0A FF", HexCodec.hex(byteArrayOf(0x01, 0x0A, 0xFF.toByte())))
    }

    @Test
    fun rendersEmptyMarker() {
        assertEquals("(empty)", HexCodec.hex(byteArrayOf()))
    }
}
