import React from "react";
import { render } from "@testing-library/react";
import App from "./App";

jest.mock("automerge-wasm/web", () => {
  const makeDoc = () => ({
    materialize: () => ({}),
    putObject: () => "edits",
    save: () => new Uint8Array(),
    splice: () => undefined,
    text: () => "the quick fox jumps over the lazy dog",
  });

  return { create: makeDoc, load: makeDoc };
});

test("renders the editor", () => {
  const { getByRole } = render(<App />);
  expect(getByRole("textbox")).toBeInTheDocument();
});
