package io.github.yeetingwaterbottle.stylo2.bleinspector.ble

/** Pure hex <-> bytes helpers, kept free of Android types so they unit-test on the JVM. */
object HexCodec {

    /** Parse a hex string ("0A 1B", "0a1b", "0x0a1b") into bytes, or null if malformed. */
    fun parseHex(input: String): ByteArray? {
        val cleaned = input.filterNot { it.isWhitespace() }.removePrefix("0x").removePrefix("0X")
        if (cleaned.isEmpty() || cleaned.length % 2 != 0) return null
        return runCatching {
            ByteArray(cleaned.length / 2) {
                cleaned.substring(it * 2, it * 2 + 2).toInt(16).toByte()
            }
        }.getOrNull()
    }

    /** Render bytes as uppercase space-separated hex. */
    fun hex(bytes: ByteArray): String =
        if (bytes.isEmpty()) "(empty)" else bytes.joinToString(" ") { "%02X".format(it) }
}
