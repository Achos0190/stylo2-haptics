package io.github.yeetingwaterbottle.stylo2.bleinspector

import io.github.yeetingwaterbottle.stylo2.bleinspector.ble.StylusHeuristics
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Test

class StylusHeuristicsTest {
    @Test
    fun flagsOnePlusStylus() {
        assertNotNull(StylusHeuristics.candidateReason("OnePlus Stylo 2", emptyList()))
    }

    @Test
    fun flagsPencil() {
        assertNotNull(StylusHeuristics.candidateReason("OPPO Pencil", emptyList()))
    }

    @Test
    fun ignoresUnrelatedDevice() {
        assertNull(StylusHeuristics.candidateReason("Mi Band 7", emptyList()))
    }

    @Test
    fun ignoresMissingName() {
        assertNull(StylusHeuristics.candidateReason(null, emptyList()))
    }
}
