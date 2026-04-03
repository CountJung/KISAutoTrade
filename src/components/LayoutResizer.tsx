/**
 * LayoutResizer — 드래그 가능한 리사이저 컴포넌트
 *
 * Holy Grail 레이아웃의 영역 크기를 조절하는 데 사용됩니다.
 * 드래그 완료(onResizeEnd) 시 localStorage에 저장합니다.
 */

import { Box } from "@mui/material";
import { useCallback, useEffect, useRef } from "react";

export type ResizeDirection = "horizontal" | "vertical";

interface LayoutResizerProps {
  direction: ResizeDirection;
  onResize: (delta: number) => void;
  onResizeEnd?: () => void;
}

export function LayoutResizer({
  direction,
  onResize,
  onResizeEnd,
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

  return (
    <Box
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
    />
  );
}
