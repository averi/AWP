// Copyright: (c) 2025, Andrea Veri <andrea.veri@gmail.com>
// GNU General Public License v3.0+ (see COPYING or https://www.gnu.org/licenses/gpl-3.0.txt)

import React from 'react';
import { AlertCircle } from 'lucide-react';

export const ErrorMessage = ({ message }) => (
  <div className="p-4 bg-red-50 border border-red-200 rounded-md flex items-center my-2">
    <AlertCircle className="w-5 h-5 text-red-500 mr-2 flex-shrink-0" />
    <p className="text-sm text-red-700">{message || 'An unknown error occurred'}</p>
  </div>
);