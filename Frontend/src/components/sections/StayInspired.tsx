'use client';

import React from 'react';
import { motion } from 'framer-motion';
import Image from 'next/image';

const cards = [
    {
        title: 'AI Market Brief',
        description: 'Get daily AI-powered summaries of crypto trends, price movements, and key market signals—explained in simple, beginner-friendly language.',
        icon: '/stay-inspired/ai_market_logo.jpg',
        bgImage: '/stay-inspired/ai_body.jpg',
        bgColor: 'bg-[#12123B]',
        textColor: 'text-white/70',
        titleColor: 'text-white',
        glowColor: 'rgba(168, 85, 247, 0.6)',
    },
    {
        title: 'Smart Learning Paths',
        description: 'Master crypto from beginner to pro with AI-guided lessons and instant feedback.',
        icon: '/stay-inspired/smart_logo.jpg',
        bgImage: '/stay-inspired/smart_body.jpg',
        bgColor: 'bg-[#D1E9F6]',
        textColor: 'text-gray-800',
        titleColor: 'text-gray-900',
        glowColor: 'rgba(56, 189, 248, 0.6)',
    },
    {
        title: 'Trade with Confidence',
        description: 'Turn knowledge into action with secure, seamless Stellar-based trading tools that bridge learning and real-world crypto activity.',
        icon: '/stay-inspired/trade_logo.jpg',
        bgImage: '/stay-inspired/trade_body.jpg',
        bgColor: 'bg-[#12123B]',
        textColor: 'text-white/70',
        titleColor: 'text-white',
        glowColor: 'rgba(234, 179, 8, 0.4)',
    }
];

const StayInspired = () => {
    return (
        <section className="py-24 px-4 bg-black overflow-hidden font-sans">
            <div className="max-w-7xl mx-auto">
                <div className="mb-16">
                    <motion.h2
                        initial={{ opacity: 0, y: 20 }}
                        whileInView={{ opacity: 1, y: 0 }}
                        viewport={{ once: true }}
                        className="text-4xl md:text-5xl font-serif text-white mb-6"
                    >
                        <span className="text-[#D946EF]">Stay Inspired</span> with our latest <br />
                        insights with <span className="text-[#D946EF]">Stellara Ai.</span>
                    </motion.h2>
                    <motion.p
                        initial={{ opacity: 0, y: 20 }}
                        whileInView={{ opacity: 1, y: 0 }}
                        viewport={{ once: true }}
                        transition={{ delay: 0.1 }}
                        className="text-lg text-gray-400 max-w-2xl leading-relaxed"
                    >
                        Stay ahead with AI-generated market intelligence, educational tips, and real-time Stellar ecosystem updates.
                    </motion.p>
                </div>

                <div className="grid md:grid-cols-3 gap-12">
                    {cards.map((card, index) => (
                        <motion.div
                            key={index}
                            initial={{ opacity: 0, y: 30 }}
                            whileInView={{ opacity: 1, y: 0 }}
                            viewport={{ once: true }}
                            transition={{ delay: index * 0.2 }}
                            className={`relative group rounded-[40px] p-10 h-auto min-h-[550px] flex flex-col ${card.bgColor} border border-white/20 transition-all duration-700 hover:scale-[1.03] overflow-visible`}
                            style={{
                                boxShadow: `0 0 100px -15px ${card.glowColor}`,
                            }}
                        >
                            {/* Massive Outer Glow (Backlight) */}
                            <div
                                className="absolute inset-0 rounded-[40px] opacity-40 group-hover:opacity-80 transition-opacity duration-700 blur-[120px] -z-10 pointer-events-none"
                                style={{ backgroundColor: card.glowColor }}
                            />

                            {/* Icon Container */}
                            <div className="w-16 h-16 rounded-full bg-black/60 backdrop-blur-2xl flex items-center justify-center mb-8 border border-white/30 shrink-0 relative z-10 shadow-xl">
                                <div className="w-10 h-10 relative">
                                    <Image
                                        src={card.icon}
                                        alt=""
                                        fill
                                        className="object-contain rounded-full"
                                    />
                                </div>
                            </div>

                            {/* Title */}
                            <h3 className={`text-3xl font-serif ${card.titleColor} mb-8 leading-tight relative z-10 font-bold`}>
                                {card.title}
                            </h3>

                            {/* Body Image (Horizontal Strip) */}
                            <div className="relative w-full h-32 mb-8 rounded-[24px] overflow-hidden border border-white/10 shadow-2xl z-10 shrink-0">
                                <Image
                                    src={card.bgImage}
                                    alt=""
                                    fill
                                    className="object-cover"
                                />
                                {/* Image Overlay Glow */}
                                <div
                                    className="absolute inset-0 opacity-20 group-hover:opacity-40 transition-opacity"
                                    style={{ background: `linear-gradient(to right, ${card.glowColor}, transparent)` }}
                                />
                            </div>

                            {/* Description */}
                            <p className={`text-xl leading-relaxed ${card.textColor} relative z-10 font-medium mb-4`}>
                                {card.description}
                            </p>

                            <div className="flex-grow" />

                            {/* Extra Glow Orb */}
                            <div
                                className="absolute -bottom-10 -right-10 w-48 h-48 rounded-full blur-[90px] opacity-20 group-hover:opacity-50 transition-all duration-700 pointer-events-none"
                                style={{ backgroundColor: card.glowColor }}
                            />
                        </motion.div>
                    ))}
                </div>
            </div>
        </section>
    );
};

export default StayInspired;
