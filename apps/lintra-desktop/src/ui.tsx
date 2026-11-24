// Shared UI primitives (T-MIG-P6-01, DEC-MIG-08): Button, Card and Badge in
// the shadcn/ui pattern (class-variance-authority + tailwind-merge + Radix
// Slot), adapted to the Lintra tokens of tokens.css. Purely presentational:
// no state, no IPC, no text of their own.
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import { clsx, type ClassValue } from "clsx";
import type { ComponentProps } from "react";
import { twMerge } from "tailwind-merge";

/** Joins class lists; later Tailwind utilities win over earlier ones. */
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}

const button = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-ctl font-semibold transition-colors "
  + "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent "
  + "aria-disabled:cursor-not-allowed aria-disabled:opacity-60 [&_svg]:size-4 [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        primary: "bg-accent text-on-accent hover:bg-accent/90",
        secondary: "border border-line bg-surface text-ink hover:bg-subtle",
        ghost: "text-ink hover:bg-subtle",
      },
      size: { md: "h-10 px-4 text-sm", sm: "h-8 px-3 text-sm", icon: "size-8" },
    },
    defaultVariants: { variant: "secondary", size: "md" },
  },
);

/**
 * Button. Unavailable actions use aria-disabled (still focusable, so the
 * reason next to them stays reachable), as the existing forms do.
 */
export function Button({ className, variant, size, asChild = false, ...props }:
  ComponentProps<"button"> & VariantProps<typeof button> & { asChild?: boolean }) {
  const Component = asChild ? Slot : "button";
  return <Component className={cn(button({ variant, size }), className)} {...props} />;
}

/** White card with the reference border, radius and soft shadow. */
export function Card({ className, ...props }: ComponentProps<"section">) {
  return <section className={cn("rounded-card border border-line bg-surface shadow-card", className)} {...props} />;
}

const badge = cva("inline-flex items-center rounded-md px-2 py-0.5 text-xs font-semibold whitespace-nowrap", {
  variants: {
    tone: {
      neutral: "bg-subtle text-muted",
      accent: "bg-selected text-selected-ink",
      error: "bg-error-soft text-error",
      warn: "bg-warn-soft text-warn",
      success: "bg-success-soft text-success",
    },
  },
  defaultVariants: { tone: "neutral" },
});

/** Status pill. The text always states the condition; colour only repeats it. */
export function Badge({ className, tone, ...props }: ComponentProps<"span"> & VariantProps<typeof badge>) {
  return <span className={cn(badge({ tone }), className)} {...props} />;
}
