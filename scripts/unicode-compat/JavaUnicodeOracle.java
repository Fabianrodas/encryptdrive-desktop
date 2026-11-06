import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Random;
import java.util.TreeMap;
import java.util.UUID;

/** Synthetic Oracle JDK 21 behavior oracle; emits API results, not copied JDK tables or source. */
public final class JavaUnicodeOracle {
    private static final int MAX = Character.MAX_CODE_POINT;
    private static final long RANDOM_SEED = 0x454e435259505444L;

    private JavaUnicodeOracle() { }

    public static void main(String[] args) {
        if (args.length != 1) {
            throw new IllegalArgumentException("Expected --mapping or --vectors");
        }
        metadata();
        switch (args[0]) {
            case "--mapping" -> emitMapping();
            case "--vectors" -> emitVectors();
            default -> throw new IllegalArgumentException("Unknown oracle mode");
        }
    }

    private static void metadata() {
        System.out.printf("META|%s|%s|Unicode-15.0%n",
                System.getProperty("java.vendor"),
                System.getProperty("java.runtime.version"));
    }

    private static void emitMapping() {
        int count = 0;
        long checksum = 0xcbf29ce484222325L;
        for (int cp = 0; cp <= MAX; cp++) {
            if (isSurrogate(cp)) {
                continue;
            }
            int folded = Character.toLowerCase(Character.toUpperCase(cp));
            if (folded != cp) {
                System.out.printf("MAP|%X|%X%n", cp, folded);
                checksum = checksumInt(checksum, cp);
                checksum = checksumInt(checksum, folded);
                count++;
            }
        }
        System.out.printf("TABLE|%d|%016X%n", count, checksum);
    }

    private static long checksumInt(long checksum, int value) {
        for (int shift = 0; shift < 32; shift += 8) {
            checksum ^= (value >>> shift) & 0xff;
            checksum *= 0x100000001b3L;
        }
        return checksum;
    }

    private static void emitVectors() {
        StringBuilder whitespace = new StringBuilder();
        Map<Integer, List<Integer>> classes = new TreeMap<>();
        int mapped = 0;
        int whitespaceCount = 0;
        for (int cp = 0; cp <= MAX; cp++) {
            if (isSurrogate(cp)) {
                continue;
            }
            if (Character.isWhitespace(cp)) {
                appendCsv(whitespace, cp);
                whitespaceCount++;
            }
            int folded = Character.toLowerCase(Character.toUpperCase(cp));
            if (folded != cp) {
                mapped++;
                List<Integer> members = classes.computeIfAbsent(folded, ignored -> new ArrayList<>());
                members.add(cp);
                if (!members.contains(folded)) {
                    members.add(folded);
                }
            }
        }
        System.out.printf("W|%s%n", whitespace);

        emitStrip("\u001c \u2000EncryptDrive\u200a\u001f");
        emitStrip("\u00a0Name\u00a0");
        emitStrip("\u2007Name\u202f");
        emitStrip("\u0085Name\u0085");
        emitStrip(" \u00a0 \u001cName\u001f \u00a0 ");
        emitStrip(" \u2000\u001c");
        emitStrip("\ud83d\ude80 \u2000Name \ud83d\ude80");

        for (String input : List.of(
                "", "a", "a".repeat(254), "a".repeat(255), "a".repeat(256),
                "a".repeat(254) + "\ud83e\udd80", "a".repeat(255) + "\ud83e\udd80",
                "\u00e9", "e\u0301", "\ud83e\udd80", "a\u0301",
                " \u001cName\u2000", ".", "..", "a" + (char) 0 + "b", "/")) {
            emitName(input);
        }

        for (int index = 0; index < 16; index++) {
            UUID identifier = new UUID(0x550e8400e29b0000L ^ index, 0x8000000000000000L ^ (index * 0x101L));
            System.out.printf("U|%s%n", identifier);
        }

        int queries = 0;
        String[][] named = {
                {"Photos", "PHOTOS"}, {"name", "NAME"}, {"\u00df", "SS"},
                {"I", "\u0131"}, {"\u0130", "i"}, {"\u0130", "i\u0307"},
                {"\u03a3", "\u03c2"}, {"\u03a3", "\u03c3"},
                {"\u212a", "K"}, {"\u00e9", "e\u0301"},
                {"\ud801\udc00", "\ud801\udc28"},
                {"\ud801\udc00A", "\ud801\udc28a"},
                {"\ud801\udc00", "A"}, {"\ufb00", "FF"}
        };
        for (String[] pair : named) {
            emitEquality(pair[0], pair[1]);
            queries++;
        }

        // Exercise every non-identity Java simple-case mapping and every mapped scalar against
        // a deterministic adjacent scalar. This tests supplementary and width-changing mappings.
        for (int cp = 0; cp <= MAX; cp++) {
            if (isSurrogate(cp)) {
                continue;
            }
            int folded = Character.toLowerCase(Character.toUpperCase(cp));
            if (folded != cp) {
                emitEquality(new String(Character.toChars(cp)), new String(Character.toChars(folded)));
                emitEquality(new String(Character.toChars(cp)), new String(Character.toChars(nextScalar(cp))));
                queries += 2;
            }
        }

        // Check all members of nontrivial equivalence classes pairwise, not just against one
        // chosen representative.
        for (List<Integer> members : classes.values()) {
            if (members.size() > 1) {
                for (int left = 0; left < members.size(); left++) {
                    for (int right = left + 1; right < members.size(); right++) {
                        emitEquality(scalar(members.get(left)), scalar(members.get(right)));
                        queries++;
                    }
                }
            }
        }

        Random random = new Random(RANDOM_SEED);
        List<List<Integer>> nontrivial = classes.values().stream()
                .filter(members -> members.size() > 1)
                .toList();
        for (int index = 0; index < 10_000; index++) {
            String left;
            String right;
            if ((index & 1) == 0) {
                List<Integer> members = nontrivial.get(random.nextInt(nontrivial.size()));
                left = scalar(members.get(random.nextInt(members.size())));
                right = scalar(members.get(random.nextInt(members.size())));
            } else {
                left = randomValidString(random);
                right = randomValidString(random);
            }
            emitEquality(left, right);
            queries++;
        }

        System.out.printf("SUMMARY|validScalars=%d|whitespace=%d|mapped=%d|queries=%d|seed=%016X%n",
                0x110000 - 0x800, whitespaceCount, mapped, queries, RANDOM_SEED);
    }

    private static void emitStrip(String input) {
        System.out.printf("S|%s|%s%n", codePointCsv(input), codePointCsv(input.strip()));
    }

    private static void emitName(String input) {
        String candidate = input.strip();
        boolean valid = !candidate.isEmpty()
                && !candidate.equals(".")
                && !candidate.equals("..")
                && candidate.indexOf('\0') < 0
                && candidate.codePointCount(0, candidate.length()) <= 255;
        System.out.printf("N|%s|%s|%s%n", codePointCsv(input), valid ? "OK" : "ERR",
                valid ? codePointCsv(candidate) : "-");
    }

    private static void emitEquality(String left, String right) {
        System.out.printf("Q|%s|%s|%s%n",
                codePointCsv(left), codePointCsv(right), left.equalsIgnoreCase(right));
    }

    private static String randomValidString(Random random) {
        int length = 1 + random.nextInt(5);
        StringBuilder result = new StringBuilder();
        for (int index = 0; index < length; index++) {
            int cp = randomScalar(random);
            result.appendCodePoint(cp);
        }
        return result.toString();
    }

    private static int randomScalar(Random random) {
        int cp;
        do {
            cp = random.nextInt(MAX + 1);
        } while (isSurrogate(cp));
        return cp;
    }

    private static int nextScalar(int cp) {
        int next = cp + 1;
        if (isSurrogate(next)) {
            next = Character.MAX_SURROGATE + 1;
        }
        if (next > MAX) {
            next = 0;
        }
        return next;
    }

    private static String scalar(int cp) {
        return new String(Character.toChars(cp));
    }

    private static String codePointCsv(String value) {
        StringBuilder result = new StringBuilder();
        value.codePoints().forEach(cp -> appendCsv(result, cp));
        return result.toString();
    }

    private static void appendCsv(StringBuilder target, int cp) {
        if (!target.isEmpty()) {
            target.append(',');
        }
        target.append(Integer.toHexString(cp));
    }

    private static boolean isSurrogate(int cp) {
        return cp >= Character.MIN_SURROGATE && cp <= Character.MAX_SURROGATE;
    }
}
