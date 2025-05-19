// Copyright: (c) 2025, Andrea Veri <andrea.veri@gmail.com>
// GNU General Public License v3.0+ (see COPYING or https://www.gnu.org/licenses/gpl-3.0.txt)

import React from 'react';

// Main Card component
export const Card = ({ children, className = '' }) => (
  <div className={`bg-white border border-gray-200 rounded-lg shadow-sm p-6 ${className}`}>
    {children}
  </div>
);

// Card Header sub-component
export const CardHeader = ({ children, className = '' }) => (
  <div className={`mb-4 ${className}`}>
    {children}
  </div>
);

// Card Title sub-component
export const CardTitle = ({ children, className = '' }) => (
  <h3 className={`text-lg font-semibold text-gray-800 flex items-center ${className}`}>
    {children}
  </h3>
);

// Card Content sub-component
export const CardContent = ({ children, className = '' }) => (
  <div className={`text-sm text-gray-600 ${className}`}>
    {children}
  </div>
);