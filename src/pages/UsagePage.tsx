import { useState, useEffect, useCallback } from 'react';
import Typography from '@mui/material/Typography';
import Table from '@mui/material/Table';
import TableHead from '@mui/material/TableHead';
import TableRow from '@mui/material/TableRow';
import TableCell from '@mui/material/TableCell';
import TableBody from '@mui/material/TableBody';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import CircularProgress from '@mui/material/CircularProgress';
import Box from '@mui/material/Box';
import CalendarTodayIcon from '@mui/icons-material/CalendarToday';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from '../i18n';

interface DailyUsage {
  date: string;
  model: string;
  provider_id: string;
  request_count: number;
  prompt_tokens: number;
  completion_tokens: number;
  cost: number;
}

interface UsageSummary {
  total_requests: number;
  total_prompt_tokens: number;
  total_completion_tokens: number;
  total_cost: number;
}

export default function UsagePage() {
  const { t } = useI18n();

  const today = new Date().toISOString().split('T')[0];
  const fifteenDaysAgo = new Date(Date.now() - 15 * 86400000).toISOString().split('T')[0];

  const [startDate, setStartDate] = useState(fifteenDaysAgo);
  const [endDate, setEndDate] = useState(today);
  const [daily, setDaily] = useState<DailyUsage[]>([]);
  const [summary, setSummary] = useState<UsageSummary>({
    total_requests: 0,
    total_prompt_tokens: 0,
    total_completion_tokens: 0,
    total_cost: 0,
  });
  const [loading, setLoading] = useState(false);

  const fetchUsage = useCallback(async () => {
    setLoading(true);
    try {
      const result = await invoke<{ daily: DailyUsage[]; summary: UsageSummary }>('get_usage', {
        params: {
          start_date: startDate,
          end_date: endDate,
        },
      });
      setDaily(result.daily || []);
      setSummary(result.summary || { total_requests: 0, total_prompt_tokens: 0, total_completion_tokens: 0, total_cost: 0 });
    } catch (e) {
      console.error('Failed to fetch usage', e);
    } finally {
      setLoading(false);
    }
  }, [startDate, endDate]);

  useEffect(() => {
    fetchUsage();
  }, [fetchUsage]);

  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
      <Card variant="outlined" sx={{ borderRadius: 2 }}>
        <CardContent>
          <Typography variant="h4" sx={{ fontWeight: 700, mb: 3, pb: 2, borderBottom: '1px solid', borderColor: 'divider', letterSpacing: -0.5 }}>{t.usage.title}</Typography>

          <Box sx={{ display: 'flex', gap: 2, alignItems: 'center', flexWrap: 'wrap', mb: 3 }}>
            <CalendarTodayIcon sx={{ color: 'text.secondary' }} />
            <Box
              component="input"
              type="date"
              value={startDate}
              onChange={(e: any) => setStartDate(e.target.value)}
              sx={{
                px: 1.5,
                py: 1,
                borderRadius: 1,
                border: '1px solid',
                borderColor: 'divider',
                bgcolor: 'background.paper',
                color: 'text.primary',
                fontFamily: 'inherit',
              }}
            />
            <Typography color="text.secondary">-</Typography>
            <Box
              component="input"
              type="date"
              value={endDate}
              onChange={(e: any) => setEndDate(e.target.value)}
              sx={{
                px: 1.5,
                py: 1,
                borderRadius: 1,
                border: '1px solid',
                borderColor: 'divider',
                bgcolor: 'background.paper',
                color: 'text.primary',
                fontFamily: 'inherit',
              }}
            />
          </Box>

          <Box
            sx={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fill, minmax(180px, 1fr))',
              gap: 2,
              mb: 3,
            }}
          >
            <Card variant="outlined" sx={{ borderRadius: 2 }}>
              <CardContent>
                <Typography variant="h4" sx={{ fontWeight: 700, color: 'primary.main' }}>{summary.total_requests}</Typography>
                <Typography variant="caption" color="text.secondary">{t.usage.totalRequests}</Typography>
              </CardContent>
            </Card>
            <Card variant="outlined" sx={{ borderRadius: 2 }}>
              <CardContent>
                <Typography variant="h4" sx={{ fontWeight: 700, color: 'primary.main' }}>{summary.total_prompt_tokens.toLocaleString()}</Typography>
                <Typography variant="caption" color="text.secondary">{t.usage.totalPromptTokens}</Typography>
              </CardContent>
            </Card>
            <Card variant="outlined" sx={{ borderRadius: 2 }}>
              <CardContent>
                <Typography variant="h4" sx={{ fontWeight: 700, color: 'primary.main' }}>{summary.total_completion_tokens.toLocaleString()}</Typography>
                <Typography variant="caption" color="text.secondary">{t.usage.totalCompletionTokens}</Typography>
              </CardContent>
            </Card>
            <Card variant="outlined" sx={{ borderRadius: 2 }}>
              <CardContent>
                <Typography variant="h4" sx={{ fontWeight: 700, color: 'primary.main' }}>${summary.total_cost.toFixed(4)}</Typography>
                <Typography variant="caption" color="text.secondary">{t.usage.totalCost}</Typography>
              </CardContent>
            </Card>
          </Box>

          <Typography variant="subtitle1" sx={{ fontWeight: 600, mb: 2 }}>{t.usage.dailyBreakdown}</Typography>

          {loading ? (
            <CircularProgress sx={{ display: 'block', mx: 'auto', my: 4 }} />
          ) : (
            <Table size="small">
              <TableHead>
                <TableRow>
                  <TableCell>{t.usage.dateRange}</TableCell>
                  <TableCell>{t.usage.model}</TableCell>
                  <TableCell>{t.usage.requestCount}</TableCell>
                  <TableCell>{t.usage.promptTokens}</TableCell>
                  <TableCell>{t.usage.completionTokens}</TableCell>
                  <TableCell>{t.usage.cost}</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {daily.map((item, idx) => (
                  <TableRow key={idx} hover>
                    <TableCell>{item.date}</TableCell>
                    <TableCell>{item.model}</TableCell>
                    <TableCell>{item.request_count}</TableCell>
                    <TableCell>{item.prompt_tokens.toLocaleString()}</TableCell>
                    <TableCell>{item.completion_tokens.toLocaleString()}</TableCell>
                    <TableCell>${item.cost.toFixed(4)}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </Box>
  );
}
