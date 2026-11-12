import { afterEach, beforeEach, expect, test } from "vitest";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  within,
} from "@testing-library/react";
import { App } from "./App";
import { load } from "./model";
beforeEach(() => localStorage.clear());
afterEach(cleanup);
test("creates and edits a task, moves it across the board, and restores it after remount", () => {
  const view = render(<App />);
  fireEvent.click(screen.getByRole("button", { name: "＋ New task" }));
  const dialog = screen.getByRole("dialog");
  fireEvent.change(within(dialog).getByLabelText("Task title"), {
    target: { value: "Book the room" },
  });
  fireEvent.change(within(dialog).getByLabelText("Notes"), {
    target: { value: "For twelve people" },
  });
  fireEvent.click(within(dialog).getByRole("button", { name: "Save task" }));
  expect(screen.getByRole("button", { name: "Book the room" })).toBeTruthy();
  fireEvent.change(screen.getByLabelText("Status for Book the room"), {
    target: { value: "doing" },
  });
  expect(
    load(localStorage).tasks.find((t) => t.title === "Book the room")?.status,
  ).toBe("doing");
  view.unmount();
  render(<App />);
  expect(screen.getByRole("button", { name: "Book the room" })).toBeTruthy();
  fireEvent.change(screen.getByPlaceholderText("Search titles and notes"), {
    target: { value: "twelve" },
  });
  expect(screen.queryByRole("button", { name: "Pick a date" })).toBeNull();
});
test("creates a project and scopes new work to it", () => {
  render(<App />);
  fireEvent.change(screen.getByLabelText("New project"), {
    target: { value: "A new venture" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Create project" }));
  expect(screen.getByRole("heading", { level: 1 }).textContent).toBe(
    "A new venture",
  );
  expect(
    load(localStorage).projects.some((p) => p.name === "A new venture"),
  ).toBe(true);
});
test("corrupt stored data blocks writes and offers recovery", () => {
  localStorage.setItem("planboard.v1", "broken");
  render(<App />);
  expect(screen.getByRole("alert").textContent).toContain("could not be read");
  expect(
    (screen.getByRole("button", { name: "＋ New task" }) as HTMLButtonElement)
      .disabled,
  ).toBe(true);
  expect(localStorage.getItem("planboard.v1")).toBe("broken");
});
