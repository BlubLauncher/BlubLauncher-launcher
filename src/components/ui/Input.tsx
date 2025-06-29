"use client";

import type React from "react";
import { forwardRef, useRef, useState } from "react";
import { Icon } from "@iconify/react";
import { cn } from "../../lib/utils";
import { useThemeStore } from "../../store/useThemeStore";

export interface InputProps
  extends Omit<React.InputHTMLAttributes<HTMLInputElement>, "size"> {
  icon?: React.ReactNode;
  clearable?: boolean;
  onClear?: () => void;
  error?: string;
  size?: "sm" | "md" | "lg";
  variant?: "default" | "flat" | "3d";
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  (
    {
      className,
      icon,
      clearable = false,
      onClear,
      error,
      size = "md",
      variant = "default",
      ...props
    },
    ref,
  ) => {
    const [isFocused, setIsFocused] = useState(false);
    const [isHovered, setIsHovered] = useState(false);
    const containerRef = useRef<HTMLDivElement>(null);
    const accentColor = useThemeStore((state) => state.accentColor);

    const handleFocus = () => {
      if (props.disabled) return;
      setIsFocused(true);
    };

    const handleBlur = () => {
      if (props.disabled) return;
      setIsFocused(false);
    };

    const handleMouseEnter = () => {
      if (props.disabled) return;
      setIsHovered(true);
    };

    const handleMouseLeave = () => {
      if (props.disabled) return;
      setIsHovered(false);
    };

    const handleClear = () => {
      if (onClear) {
        onClear();
      } else if (props.onChange) {
        const event = {
          target: { value: "" },
        } as React.ChangeEvent<HTMLInputElement>;
        props.onChange(event);
      }
    };

    const sizeClasses = {
      sm: "h-[42px]",
      md: "h-[50px]",
      lg: "h-[58px]",
    };

    const inputSizeClasses = {
      sm: "text-xl",
      md: "text-2xl",
      lg: "text-3xl",
    };

    // Get border classes based on variant
    const getBorderClasses = () => {
      if (variant === "3d") {
        return "border-2 border-b-4";
      }
      return "border border-b-2";
    };

    return (
      <div className="w-full">
        <div
          ref={containerRef}
          className={cn(
            "relative rounded-md",
            getBorderClasses(),
            "overflow-hidden",
            error ? "border-red-500" : "",
            props.disabled ? "opacity-50 cursor-not-allowed" : "",
            sizeClasses[size],
            className,
          )}
          style={{
            backgroundColor: `${accentColor.value}${isHovered || isFocused ? "50" : "30"}`,
            borderColor: error
              ? "rgba(239, 68, 68, 0.6)"
              : `${accentColor.value}${isHovered || isFocused ? "90" : "80"}`,
            borderBottomColor: error
              ? "rgb(185, 28, 28)"
              : isHovered || isFocused
                ? accentColor.hoverValue
                : accentColor.value,
            boxShadow:
              variant === "3d"
                ? `0 4px 0 rgba(0,0,0,0.3), 0 6px 10px rgba(0,0,0,0.35)`
                : "none",
            filter:
              (isFocused || isHovered) && !props.disabled
                ? "brightness(1.1)"
                : "brightness(1)",
          }}
          onMouseEnter={handleMouseEnter}
          onMouseLeave={handleMouseLeave}
        >
          {variant === "3d" && (
            <span
              className="absolute inset-x-0 top-0 h-[2px] rounded-t-sm transition-colors duration-200"
              style={{
                backgroundColor: error
                  ? "rgba(239, 68, 68, 0.8)"
                  : isHovered || isFocused
                    ? accentColor.hoverValue
                    : `${accentColor.value}80`,
                opacity: isHovered || isFocused ? 1 : 0.8,
              }}
            />
          )}

          <div className="flex items-center h-full w-full">
            {icon && (
              <div className="flex items-center justify-center w-10 h-full text-white">
                {icon}
              </div>
            )}

            <input
              ref={ref}
              className={cn(
                "flex-1 h-full bg-transparent border-none outline-none px-3 py-2 text-white font-minecraft placeholder:text-white/50 lowercase",
                inputSizeClasses[size],
              )}
              onFocus={handleFocus}
              onBlur={handleBlur}
              spellCheck={false}
              autoComplete="off"
              {...props}
            />

            {clearable && props.value && (
              <button
                type="button"
                onClick={handleClear}
                className="flex items-center justify-center w-10 h-full transition-opacity duration-200 hover:opacity-80 text-white"
                tabIndex={-1}
              >
                <Icon icon="solar:close-circle-bold" className="w-4 h-4" />
              </button>
            )}
          </div>
        </div>

        {error && (
          <p className="mt-1 text-xl text-red-400 font-minecraft lowercase">
            {error}
          </p>
        )}
      </div>
    );
  },
);

Input.displayName = "Input";
