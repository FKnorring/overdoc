# OverDoc Analysis Results

## Repository: ../footstat

## Summary

- Total files analyzed: 219
- Total exported entities: 345
- Files with exports: 141
- Total lines of code: 19713
- Code lines: 17485
- Comment lines: 567
- Blank lines: 1661
- Comment ratio: 3.14%
- Average lines per file: 90
- Average cyclomatic complexity: 5.92
- Average cognitive complexity: 9.96
- Average maintainability index: 66.03

### Language Distribution

- tsx: 118 files (53.9%)
- ts: 100 files (45.7%)
- js: 1 files (0.5%)

### Knowledge Hotspots

Files with highest knowledge scores (combining complexity, size, and importance):

1. **../footstat\components\pages\props\stats\MainStats.tsx** (Knowledge Score: 67.2)
2. **../footstat\lib\minifyer.ts** (Knowledge Score: 64.3)
3. **../footstat\components\pages\props\stats\player\SupportingStats.tsx** (Knowledge Score: 51.0)
4. **../footstat\lib\odds.ts** (Knowledge Score: 50.2)
5. **../footstat\components\pages\props\fixtures\FixtureList.tsx** (Knowledge Score: 49.2)

## Top Important Files

1. **../footstat\lib\utils.ts** (Score: 111)
   - function `cn` (used 34 times)
   - function `capitalize` (used 2 times)
   - function `getPlayerImageUrl` (used 3 times)
   - Lines: 18 (Code: 15, Comments: 0, Blank: 3)
   - Functions: 3, Comment ratio: 0.0%
   - Complexity: Cyclomatic: 3.0, Cognitive: 7.5, Maintainability: 89.5 (Cyclomatic: 3.0, Cognitive: 7.5)
   - Maintainability Index: 89.5 (Higher is better)
   - Knowledge Score: 22.9

2. **../footstat\lib\utils\class-names.ts** (Score: 102)
   - function `cn` (used 34 times)
   - Lines: 6 (Code: 5, Comments: 0, Blank: 1)
   - Functions: 1, Comment ratio: 0.0%
   - Complexity: Cyclomatic: 1.0, Cognitive: 1.0, Maintainability: 100.0 (Cyclomatic: 1.0, Cognitive: 1.0)
   - Maintainability Index: 100.0 (Higher is better)
   - Knowledge Score: 15.8

3. **../footstat\models\odds.ts** (Score: 70)
   - interface `Market` (used 2 times)
   - interface `PlayerMarkets` (used 0 times)
   - interface `Odds` (used 0 times)
   - interface `OddsPerMarketPerPlayer` (used 0 times)
   - interface `OpticOddPerMarketPerPlayer` (used 0 times)
   - interface `PlayerOddsInfo` (used 0 times)
   - interface `OpticPlayerOddsInfo` (used 3 times)
   - interface `FixtureWithOdds` (used 0 times)
   - interface `FixtureOddsResponse` (used 0 times)
   - interface `PlayerWithOdds` (used 7 times)
   - interface `TeamWithOdds` (used 5 times)
   - interface `EnhancedFixtureWithOdds` (used 11 times)
   - Lines: 96 (Code: 81, Comments: 3, Blank: 12)
   - Functions: 0, Comment ratio: 3.6%
   - Declarations: interface: 12
   - Complexity: Cyclomatic: 1.0, Cognitive: 0.0, Maintainability: 57.5 (Cyclomatic: 1.0, Cognitive: 0.0)
   - Maintainability Index: 57.5 (Higher is better)
   - Knowledge Score: 31.8

4. **../footstat\models\api.ts** (Score: 28)
   - interface `League` (used 1 times)
   - interface `Season` (used 0 times)
   - interface `Participant` (used 0 times)
   - interface `APIFixture` (used 2 times)
   - interface `SeasonResponse` (used 1 times)
   - interface `FixtureResponse` (used 1 times)
   - interface `Lineup` (used 3 times)
   - interface `PlayerDetails` (used 1 times)
   - interface `FixtureStatistic` (used 1 times)
   - interface `MigrationStats` (used 0 times)
   - interface `MigrationOptions` (used 0 times)
   - interface `ProcessedTeams` (used 0 times)
   - Lines: 139 (Code: 117, Comments: 10, Blank: 12)
   - Functions: 0, Comment ratio: 7.9%
   - Declarations: interface: 13
   - Complexity: Cyclomatic: 1.0, Cognitive: 0.0, Maintainability: 50.0 (Cyclomatic: 1.0, Cognitive: 0.0)
   - Maintainability Index: 50.0 (Higher is better)
   - Knowledge Score: 28.9

5. **../footstat\app\[league]\types\teamStats.ts** (Score: 26)
   - type `StatValue` (used 0 times)
   - type `StatCategory` (used 1 times)
   - type `TeamStats` (used 3 times)
   - type `TeamWithStats` (used 6 times)
   - type `StatConfig` (used 0 times)
   - constant `STAT_CONFIGS` (used 0 times)
   - Lines: 127 (Code: 121, Comments: 1, Blank: 5)
   - Functions: 0, Comment ratio: 0.8%
   - Declarations: type: 5
   - Complexity: Cyclomatic: 2.0, Cognitive: 1.0, Maintainability: 51.9 (Cyclomatic: 2.0, Cognitive: 1.0)
   - Maintainability Index: 51.9 (Higher is better)
   - Knowledge Score: 24.3

6. **../footstat\app\context\TrialContext.tsx** (Score: 24)
   - Lines: 67 (Code: 60, Comments: 0, Blank: 7)
   - Functions: 3, Comment ratio: 0.0%
   - Declarations: interface: 2
   - Complexity: Cyclomatic: 5.0, Cognitive: 12.0, Maintainability: 63.1 (Cyclomatic: 5.0, Cognitive: 12.0)
   - Maintainability Index: 63.1 (Higher is better)
   - Knowledge Score: 22.3

7. **../footstat\config\flags.ts** (Score: 20)
   - Lines: 13 (Code: 12, Comments: 0, Blank: 1)
   - Functions: 1, Comment ratio: 0.0%
   - Complexity: Cyclomatic: 2.0, Cognitive: 3.0, Maintainability: 100.0 (Cyclomatic: 2.0, Cognitive: 3.0)
   - Maintainability Index: 100.0 (Higher is better)
   - Knowledge Score: 8.1

8. **../footstat\config\stripe.ts** (Score: 19)
   - Lines: 181 (Code: 171, Comments: 0, Blank: 10)
   - Functions: 1, Comment ratio: 0.0%
   - Declarations: interface: 1, type: 1
   - Complexity: Cyclomatic: 3.0, Cognitive: 4.0, Maintainability: 40.1 (Cyclomatic: 3.0, Cognitive: 4.0)
   - Maintainability Index: 40.1 (Higher is better)
   - Knowledge Score: 24.7

9. **../footstat\lib\api-config.ts** (Score: 18)
   - Lines: 34 (Code: 30, Comments: 0, Blank: 4)
   - Functions: 2, Comment ratio: 0.0%
   - Declarations: type: 1
   - Complexity: Cyclomatic: 7.0, Cognitive: 9.5, Maintainability: 75.1 (Cyclomatic: 7.0, Cognitive: 9.5)
   - Maintainability Index: 75.1 (Higher is better)
   - Knowledge Score: 17.2

10. **../footstat\hooks\useWindowWidth.ts** (Score: 18)
   - Lines: 23 (Code: 16, Comments: 1, Blank: 6)
   - Functions: 3, Comment ratio: 5.9%
   - Complexity: Cyclomatic: 2.0, Cognitive: 4.0, Maintainability: 88.0 (Cyclomatic: 2.0, Cognitive: 4.0)
   - Maintainability Index: 88.0 (Higher is better)
   - Knowledge Score: 12.3

## Top Important Directories

1. **..** (Score: 916)
   - Files: 219, Total lines: 19713, Functions: 697

2. **../footstat** (Score: 916)
   - Files: 219, Total lines: 19713, Functions: 697

3. **../footstat\lib** (Score: 307)
   - Files: 28, Total lines: 3131, Functions: 84

4. **../footstat\components** (Score: 272)
   - Files: 109, Total lines: 9438, Functions: 372

5. **../footstat\components\pages** (Score: 233)
   - Files: 74, Total lines: 6557, Functions: 239

6. **../footstat\components\pages\props** (Score: 170)
   - Files: 57, Total lines: 5151, Functions: 194

7. **../footstat\app** (Score: 122)
   - Files: 44, Total lines: 3697, Functions: 124

8. **../footstat\models** (Score: 116)
   - Files: 5, Total lines: 316, Functions: 0

9. **../footstat\lib\utils** (Score: 111)
   - Files: 5, Total lines: 95, Functions: 9

10. **../footstat\components\pages\props\stats** (Score: 93)
   - Files: 25, Total lines: 3204, Functions: 106

