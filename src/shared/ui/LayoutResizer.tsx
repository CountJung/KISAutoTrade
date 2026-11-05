/**
 * LayoutResizer — 드래그/키보드로 조절하는 리사이저 컴포넌트
 *
 * Holy Grail 레이아웃의 영역 크기를 조절하는 데 사용됩니다.
 * 드래그 완료(onResizeEnd) 시 localStorage에 저장합니다.
 */

import { Box } from "@mui/material";
import { useCallback, useEffect, useId, useRef } from "react";

export type ResizeDirection = "horizontal" | "vertical";

interface LayoutResizerProps {
  direction: ResizeDirection;
  onResize: (delta: number) => void;
  onResizeEnd?: () => void;
  label: string;
  value: number;
  min: number;
  max: number;
  defaultValue: number;
  onValueChange: (value: number) => void;
  keyboardStep?: number;
}

export function LayoutResizer({
  direction,
  onResize,
  onResizeEnd,
  label,
  value,
  min,
  max,
  defaultValue,
  onValueChange,
  keyboardStep = 10,
}: LayoutResizerProps) {
  const isDragging = useRef(false);
  const lastPosition = useRef(0);

  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      isDragging.current = true;
      lastPosition.current = direction === "horizontal" ? e.clientX : e.clientY;
      document.body.style.cursor =
        direction === "horizontal" ? "col-resize" : "row-resize";
      document.body.style.userSelect = "none";
    },
    [direction],
  );

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!isDragging.current) return;

      const currentPosition =
        direction === "horizontal" ? e.clientX : e.clientY;
      const delta = currentPosition - lastPosition.current;
      lastPosition.current = currentPosition;

      onResize(delta);
    };

    const handleMouseUp = () => {
      if (isDragging.current) {
        isDragging.current = false;
        document.body.style.cursor = "";
        document.body.style.userSelect = "";
        onResizeEnd?.();
      }
    };

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);

    return () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
    };
  }, [direction, onResize, onResizeEnd]);

  const isHorizontal = direction === "horizontal";
  const helpId = useId();
  const keyboardHelp = `${isHorizontal ? '좌우' : '상하'} 방향키: ${keyboardStep}픽셀 조절. Home: 최소 ${min}픽셀. End: 최대 ${max}픽셀. Enter: 기본 ${defaultValue}픽셀 복원.`;

  const handleKeyDown = (event: React.KeyboardEvent) => {
    const decrease = isHorizontal ? 'ArrowLeft' : 'ArrowUp';
    const increase = isHorizontal ? 'ArrowRight' : 'ArrowDown';
    let target: number;
    if (event.key === decrease) target = value - keyboardStep;
    else if (event.key === increase) target = value + keyboardStep;
    else if (event.key === 'Home') target = min;
    else if (event.key === 'End') target = max;
    else if (event.key === 'Enter') target = defaultValue;
    else return;
    event.preventDefault();
    onValueChange(Math.max(min, Math.min(max, target)));
  };

  return (
    <Box
      role="separator"
      tabIndex={0}
      aria-label={label}
      aria-orientation={isHorizontal ? 'vertical' : 'horizontal'}
      aria-valuemin={min}
      aria-valuemax={max}
      aria-valuenow={value}
      aria-valuetext={`${value}픽셀`}
      aria-describedby={helpId}
      title={keyboardHelp}
      onKeyDown={handleKeyDown}
      onMouseDown={handleMouseDown}
      sx={{
        position: "relative",
        width: isHorizontal ? 12 : "100%",
        height: isHorizontal ? "100%" : 12,
        cursor: isHorizontal ? "col-resize" : "row-resize",
        bgcolor: "transparent",
        transition: "background-color 0.2s",
        zIndex: 10,
        flexShrink: 0,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        "&:hover": {
          bgcolor: "action.hover",
        },
        "&:focus-visible": {
          outline: '2px solid',
          outlineColor: 'primary.main',
          outlineOffset: -2,
          bgcolor: 'action.hover',
        },
        "&:focus-visible::before": {
          bgcolor: 'primary.main',
        },
        "&::before": {
          content: '""',
          position: "absolute",
          width: isHorizontal ? 6 : 60,
          height: isHorizontal ? 60 : 6,
          bgcolor: "divider",
          borderRadius: 3,
          transition: "all 0.2s ease",
          boxShadow: 1,
        },
        "&:hover::before": {
          bgcolor: "primary.main",
          width: isHorizontal ? 8 : 80,
          height: isHorizontal ? 80 : 8,
          boxShadow: 3,
        },
        "&::after": {
          content: '""',
          position: "absolute",
          width: isHorizontal ? 2 : 24,
          height: isHorizontal ? 24 : 2,
          background: isHorizontal
            ? "repeating-linear-gradient(to bottom, rgba(255,255,255,0.5) 0px, rgba(255,255,255,0.5) 4px, transparent 4px, transparent 8px)"
            : "repeating-linear-gradient(to right, rgba(255,255,255,0.5) 0px, rgba(255,255,255,0.5) 4px, transparent 4px, transparent 8px)",
          borderRadius: 1,
          pointerEvents: "none",
        },
      }}
    >
      <Box
        component="span"
        id={helpId}
        sx={{ position: 'absolute', width: 1, height: 1, p: 0, overflow: 'hidden', clip: 'rect(0, 0, 0, 0)', whiteSpace: 'nowrap' }}
      >
        {keyboardHelp}
      </Box>
    </Box>
  );
}
