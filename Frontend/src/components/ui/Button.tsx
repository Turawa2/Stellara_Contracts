import type { ButtonHTMLAttributes } from "react";

type ButtonVariant = "primary" | "secondary" | "wallet";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
}

const variants: Record<ButtonVariant, string> = {
  primary:
    "bg-brand-blue text-black hover:opacity-90 font-inter text-base font-semibold",
  secondary:
    "bg-white text-brand-blue hover:opacity-90 font-inter text-base font-semibold",
  wallet:
    "bg-[#242424] text-white hover:bg-[#2f2f2f] font-manrope text-base font-medium",
};

export function Button({
  variant = "primary",
  className = "",
  children,
  ...props
}: ButtonProps) {
  return (
    <button
      className={`inline-flex items-center justify-center rounded-full px-6 py-2 transition duration-200 ${variants[variant]} ${className}`}
      {...props}
    >
      {children}
    </button>
  );
}
