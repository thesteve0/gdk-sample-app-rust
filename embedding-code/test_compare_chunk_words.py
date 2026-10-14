import unittest
from compare_chunk_words import page_rows, section_rows, candidates


class ComparisonTests(unittest.TestCase):
    def test_overlap_rounding_empty_reset_no_recursion(self):
        pages = [dict(number=1, raw_text="one two three"), dict(number=2, raw_text="four five"), dict(number=3, raw_text=""), dict(number=4, raw_text="six")]
        rows = page_rows(pages)
        self.assertEqual([r['total_words'] for r in rows], [3, 3, 1])
        self.assertEqual([r['overlap_words'] for r in rows], [0, 1, 0])

    def test_sections_cover_words_without_duplication(self):
        pages = [dict(number=1, raw_text="Intro\nMajor\nBody words\nMinor\nMore words\n")]
        headings = [dict(page=1, line=2, title='Major', level='major'), dict(page=1, line=4, title='Minor', level='subsection')]
        for fine, expected in [(False, 2), (True, 3)]:
            rows = section_rows(pages, headings, fine)
            self.assertEqual(len(rows), expected)
            self.assertEqual(sum(r['total_words'] for r in rows), 7)

    def test_typographic_spans_join_before_line_matching(self):
        pages = [dict(number=8, raw_text="Synthetic Major Heading\nBody\n")]
        xml = b'<pdf2xml><page number="8"><fontspec id="0" size="21" family="OpenSans" color="#122d56"/><text top="10" left="1" font="0">Synthetic Major</text><text top="10" left="100" font="0">Heading</text></page></pdf2xml>'
        hs = candidates(pages, xml, 'cfi')
        self.assertEqual(len(hs), 1)
        self.assertEqual(hs[0]['line'], 1)


if __name__ == '__main__':
    unittest.main()
