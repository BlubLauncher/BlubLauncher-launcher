"use client";

import { useEffect } from "react";
import { useThemeStore } from "../store/useThemeStore";

export function ThemeInitializer() {
  const applyAccentColorToDOM = useThemeStore(
    (state) => state.applyAccentColorToDOM,
  );

  useEffect(() => {
    applyAccentColorToDOM();
  }, [applyAccentColorToDOM]);

  return null;
}
