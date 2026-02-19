import type { Metadata } from "next";
import { HeroSection } from "@/components/sections/HeroSection";
import { Navbar } from "@/components/layout/Navbar";
import "./globals.css";

export const metadata: Metadata = {
  title: "Stellara AI",
  description: "Learn. Trade. Connect. Powered by AI on Stellar.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className="bg-background text-foreground antialiased">
        <Navbar />
        <HeroSection />
        {children}
      </body>
    </html>
  );
}
