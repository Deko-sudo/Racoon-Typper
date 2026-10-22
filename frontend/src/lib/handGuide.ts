// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

// Original SVG geometry for this project. Each highlight uses the complete
// finger contour also used by the hand outline, including its rounded tip.
export const HAND_FINGERS = [
  { code: 'P', start: '30 132', contour: 'L20 89C18 79 22 71 30 70C38 69 44 75 46 84L54 120', base: 'Q43 132 30 132Z', detail: 'M27 86Q32 81 39 85M31 108L43 105' },
  { code: 'R', start: '54 120', contour: 'L45 50C44 40 49 33 57 32C66 31 72 38 73 47L79 114', base: 'Q67 124 54 120Z', detail: 'M52 49Q58 44 66 48M56 88L70 86' },
  { code: 'M', start: '79 114', contour: 'L75 28C74 17 80 9 89 9C98 9 104 16 104 27L106 114', base: 'Q92 122 79 114Z', detail: 'M82 28Q89 23 97 28M84 78L98 78' },
  { code: 'I', start: '106 114', contour: 'L108 44C108 33 114 26 123 27C132 28 137 35 136 46L132 126', base: 'Q118 127 106 114Z', detail: 'M115 45Q122 40 129 45M114 90L128 91' },
  { code: 'T', start: '132 166', contour: 'L156 116C160 108 168 104 175 108C183 112 185 120 180 129L160 168C155 177 151 185 150 197', base: 'Q136 185 132 166Z', detail: 'M160 123Q165 117 173 122M147 151L159 158' },
] as const;

// Reuse those exact outer curves rather than maintaining separate clip bands.
export const HAND_OUTLINE = `M${HAND_FINGERS[0].start}${HAND_FINGERS.slice(0, 4).map((finger) => finger.contour).join('')}L${HAND_FINGERS[4].start}${HAND_FINGERS[4].contour}L148 222Q105 234 62 222C42 207 36 188 34 166Z`;
