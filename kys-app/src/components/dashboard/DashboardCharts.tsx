import React from "react"
import { Area, AreaChart, Bar, BarChart, CartesianGrid, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts"

const areaData = [
  { date: "Apr 5", val1: 4000, val2: 2400, val3: 2400 },
  { date: "Apr 10", val1: 3000, val2: 1398, val3: 2210 },
  { date: "Apr 15", val1: 2000, val2: 9800, val3: 2290 },
  { date: "Apr 20", val1: 2780, val2: 3908, val3: 2000 },
  { date: "Apr 25", val1: 1890, val2: 4800, val3: 2181 },
  { date: "Apr 30", val1: 2390, val2: 3800, val3: 2500 },
]

const barData = [
  { month: "Jan", val1: 4000, val2: 2400, val3: 2400 },
  { month: "Feb", val1: 3000, val2: 1398, val3: 2210 },
  { month: "Mar", val1: 2000, val2: 9800, val3: 2290 },
  { month: "Apr", val1: 2780, val2: 3908, val3: 2000 },
  { month: "May", val1: 1890, val2: 4800, val3: 2181 },
  { month: "Jun", val1: 2390, val2: 3800, val3: 2500 },
  { month: "Jul", val1: 3490, val2: 4300, val3: 2100 },
]

export function DashboardCharts() {
  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-8">
      {/* Area Chart Card */}
      <div className="bg-card rounded-xl p-6 shadow-sm border border-border flex flex-col">
        <div className="mb-6">
          <h3 className="text-sm font-medium text-muted-foreground">Proje Analiz Trendi</h3>
          <div className="flex items-center gap-3 mt-1">
            <span className="text-3xl font-bold">13,277</span>
            <span className="text-xs font-bold text-green-600 bg-green-500/10 px-2 py-1 rounded-full">+35%</span>
          </div>
          <p className="text-xs text-muted-foreground mt-1">Son 30 günün analizi</p>
        </div>
        <div className="h-64 w-full">
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart data={areaData} margin={{ top: 10, right: 0, left: -20, bottom: 0 }}>
              <defs>
                <linearGradient id="colorVal1" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.8}/>
                  <stop offset="95%" stopColor="#3b82f6" stopOpacity={0}/>
                </linearGradient>
                <linearGradient id="colorVal2" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#60a5fa" stopOpacity={0.8}/>
                  <stop offset="95%" stopColor="#60a5fa" stopOpacity={0}/>
                </linearGradient>
                <linearGradient id="colorVal3" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#93c5fd" stopOpacity={0.8}/>
                  <stop offset="95%" stopColor="#93c5fd" stopOpacity={0}/>
                </linearGradient>
              </defs>
              <XAxis dataKey="date" axisLine={false} tickLine={false} tick={{ fontSize: 12, fill: 'hsl(var(--muted-foreground))' }} dy={10} />
              <YAxis axisLine={false} tickLine={false} tick={{ fontSize: 12, fill: 'hsl(var(--muted-foreground))' }} />
              <Tooltip 
                contentStyle={{ backgroundColor: 'hsl(var(--card))', borderColor: 'hsl(var(--border))', borderRadius: '8px' }}
                itemStyle={{ color: 'hsl(var(--foreground))' }}
              />
              <Area type="monotone" dataKey="val3" stackId="1" stroke="#93c5fd" fill="url(#colorVal3)" />
              <Area type="monotone" dataKey="val2" stackId="1" stroke="#60a5fa" fill="url(#colorVal2)" />
              <Area type="monotone" dataKey="val1" stackId="1" stroke="#3b82f6" fill="url(#colorVal1)" />
            </AreaChart>
          </ResponsiveContainer>
        </div>
      </div>

      {/* Bar Chart Card */}
      <div className="bg-card rounded-xl p-6 shadow-sm border border-border flex flex-col">
        <div className="mb-6">
          <h3 className="text-sm font-medium text-muted-foreground">Kategori Analizi ve Hatalar</h3>
          <div className="flex items-center gap-3 mt-1">
            <span className="text-3xl font-bold">1.3M</span>
            <span className="text-xs font-bold text-red-600 bg-red-500/10 px-2 py-1 rounded-full">-8%</span>
          </div>
          <p className="text-xs text-muted-foreground mt-1">Son 6 aydaki dağılım</p>
        </div>
        <div className="h-64 w-full">
          <ResponsiveContainer width="100%" height="100%">
            <BarChart data={barData} margin={{ top: 10, right: 0, left: -20, bottom: 0 }}>
              <XAxis dataKey="month" axisLine={false} tickLine={false} tick={{ fontSize: 12, fill: 'hsl(var(--muted-foreground))' }} dy={10} />
              <YAxis axisLine={false} tickLine={false} tick={{ fontSize: 12, fill: 'hsl(var(--muted-foreground))' }} />
              <Tooltip 
                contentStyle={{ backgroundColor: 'hsl(var(--card))', borderColor: 'hsl(var(--border))', borderRadius: '8px' }}
                itemStyle={{ color: 'hsl(var(--foreground))' }}
                cursor={{ fill: 'hsl(var(--muted) / 0.4)' }}
              />
              <Bar dataKey="val1" stackId="a" fill="#3b82f6" radius={[0, 0, 4, 4]} />
              <Bar dataKey="val2" stackId="a" fill="#60a5fa" />
              <Bar dataKey="val3" stackId="a" fill="#93c5fd" radius={[4, 4, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </div>
    </div>
  )
}
