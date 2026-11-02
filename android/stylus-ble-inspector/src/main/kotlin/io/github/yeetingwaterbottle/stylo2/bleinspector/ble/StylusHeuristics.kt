package io.github.yeetingwaterbottle.stylo2.bleinspector.ble

/**
 * Name-based heuristics to flag a scanned device as a *candidate* Stylo 2.
 *
 * Heuristic only — it never asserts identity. No service UUIDs or MAC OUIs are
 * hard-coded, because none are verified for the Stylo 2 (see
 * `stylo2/docs/C-protocol.md`). The operator confirms the real device by
 * connecting and inspecting it.
 */
object StylusHeuristics {
    private val NAME_HINTS = listOf(
        "stylo", "stylus", "pencil", "pen", "oplus", "oppo", "oneplus", "ipe",
    )

    /** Returns a short reason when [name] looks like the pen, else null. */
    fun candidateReason(name: String?, serviceUuids: List<String>): String? {
        val lower = name?.lowercase().orEmpty()
        val hit = NAME_HINTS.firstOrNull { lower.contains(it) }
        return hit?.let { "name contains \"$it\"" }
    }
}
