# Expense Summary

Group a JSON expense file by category and total its amounts. Store amounts as integer cents so the sum stays exact.

Choose **Run** in Devline and open **App Output** to see the bundled example. In a generated project, run:

```sh
./devline.sh run
./devline.sh exec swift run ExpenseSummary examples/expenses.json
./devline.sh test
```

Copy `examples/expenses.json` and use your own categories and positive amounts. The program prints sorted category totals and a grand total. SwiftPM builds the tool with the selected local macOS SDK.
