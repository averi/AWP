// Copyright: (c) 2025, Andrea Veri <andrea.veri@gmail.com>
// GNU General Public License v3.0+ (see COPYING or https://www.gnu.org/licenses/gpl-3.0.txt)

import React from 'react';

export const Button = ({ children, onClick, variant = 'default', size = 'default', disabled = false, className = '', ...props }) => {
  const baseStyle = "inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none";

  const variants = {
    default: "bg-blue-600 text-white hover:bg-blue-700 focus:ring-blue-500",
    outline: "border border-gray-300 bg-white text-gray-700 hover:bg-gray-50 focus:ring-blue-500",
    ghost: "hover:bg-gray-100 text-gray-700 focus:ring-blue-500",
  };

  const sizes = {
    default: "px-4 py-2",
    sm: "px-3 py-1.5 text-xs",
    lg: "px-6 py-3 text-lg",
    icon: "p-2",
  };

  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className={`${baseStyle} ${variants[variant]} ${sizes[size]} ${className}`}
      {...props}
    >
      {children}
    </button>
  );
};