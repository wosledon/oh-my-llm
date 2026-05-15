import { useState, useCallback, useEffect } from 'react';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import TextField from '@mui/material/TextField';
import FormControl from '@mui/material/FormControl';
import InputLabel from '@mui/material/InputLabel';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import Table from '@mui/material/Table';
import TableHead from '@mui/material/TableHead';
import TableRow from '@mui/material/TableRow';
import TableCell from '@mui/material/TableCell';
import TableBody from '@mui/material/TableBody';
import Dialog from '@mui/material/Dialog';
import DialogTitle from '@mui/material/DialogTitle';
import DialogContent from '@mui/material/DialogContent';
import DialogActions from '@mui/material/DialogActions';
import Tabs from '@mui/material/Tabs';
import Tab from '@mui/material/Tab';
import Chip from '@mui/material/Chip';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import CircularProgress from '@mui/material/CircularProgress';
import Box from '@mui/material/Box';
import IconButton from '@mui/material/IconButton';
import SearchIcon from '@mui/icons-material/Search';
import FilterListIcon from '@mui/icons-material/FilterList';
import ChevronLeftIcon from '@mui/icons-material/ChevronLeft';
import ChevronRightIcon from '@mui/icons-material/ChevronRight';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from '../i18n';

interface LogItem {
  id: string;
  timestamp: number;
  protocol: string;
  model: string;
  upstream_model?: string;
  provider_id?: string;
  stream: boolean;
  latency_ms: number;
  status_code: number;
  prompt_tokens: number;
  completion_tokens: number;
  cost: number;
  error_type?: string;
}

export default function LogsPage() {
  const { t } = useI18n();

  const [logs, setLogs] = useState<LogItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState<string>('all');
  const [offset, setOffset] = useState(0);
  const [searchDebounce, setSearchDebounce] = useState('');
  const [selectedLog, setSelectedLog] = useState<LogItem | null>(null);
  const [detailOpen, setDetailOpen] = useState(false);
  const [detailTab, setDetailTab] = useState(0);
  const [detailBody, setDetailBody] = useState({ request: '', response: '' });

  const limit = 20;

  const fetchLogs = useCallback(async () => {
    setLoading(true);
    try {
      const result = await invoke<LogItem[]>('query_logs', {
        params: {
          search: searchDebounce || null,
          status: statusFilter === 'all' ? null : statusFilter,
          limit,
          offset,
        },
      });
      setLogs(result || []);
    } catch (e) {
      console.error('Failed to fetch logs', e);
    } finally {
      setLoading(false);
    }
  }, [searchDebounce, statusFilter, offset]);

  useEffect(() => {
    fetchLogs();
  }, [fetchLogs]);

  useEffect(() => {
    const timer = setTimeout(() => {
      setSearchDebounce(search);
      setOffset(0);
    }, 400);
    return () => clearTimeout(timer);
  }, [search]);

  const handleOpenDetail = async (log: LogItem) => {
    setSelectedLog(log);
    setDetailOpen(true);
    setDetailBody({ request: '', response: '' });
    try {
      const detail = await invoke<{ request_body: string; response_body: string }>('get_log_detail', {
        logId: log.id,
      });
      if (detail) {
        setDetailBody({
          request: detail.request_body,
          response: detail.response_body,
        });
      }
    } catch (e) {
      console.error('Failed to fetch log detail', e);
    }
  };

  const formatTime = (ts: number) => {
    return new Date(ts * 1000).toLocaleString();
  };

  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
      <Card variant="outlined" sx={{ borderRadius: 2 }}>
        <CardContent>
          <Typography variant="h4" sx={{ fontWeight: 700, mb: 3, pb: 2, borderBottom: '1px solid', borderColor: 'divider', letterSpacing: -0.5 }}>{t.logs.title}</Typography>

          <Box sx={{ display: 'flex', gap: 2, alignItems: 'center', flexWrap: 'wrap', mb: 2 }}>
            <TextField
              size="small"
              placeholder={t.logs.search}
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              slotProps={{
                input: {
                  startAdornment: <SearchIcon fontSize="small" sx={{ mr: 1, color: 'text.secondary' }} />,
                },
              }}
              onKeyDown={(e) => { if (e.key === 'Enter') { setOffset(0); fetchLogs(); }}}
            />
            <FormControl size="small" sx={{ minWidth: 120 }}>
              <InputLabel>{t.logs.filter}</InputLabel>
              <Select
                value={statusFilter}
                label={t.logs.filter}
                onChange={(e) => { setStatusFilter(e.target.value); setOffset(0); }}
                MenuProps={{
                  slotProps: {
                    paper: {
                      sx: {
                        mt: 0.5,
                        borderRadius: 2,
                        boxShadow: (theme: any) => theme.shadows[8],
                      },
                    },
                  },
                }}
              >
                <MenuItem value="all">{t.logs.all}</MenuItem>
                <MenuItem value="success">{t.logs.success}</MenuItem>
                <MenuItem value="error">{t.logs.error}</MenuItem>
              </Select>
            </FormControl>
            <IconButton size="small" sx={{ ml: 0.5 }} onClick={() => { setOffset(0); fetchLogs(); }} title={t.common.confirm}>
              <FilterListIcon fontSize="small" />
            </IconButton>
          </Box>

          {loading ? (
            <CircularProgress sx={{ display: 'block', mx: 'auto', my: 4 }} />
          ) : (
            <>
              <Table size="small">
                <TableHead>
                  <TableRow>
                    <TableCell>{t.logs.time}</TableCell>
                    <TableCell>{t.logs.model}</TableCell>
                    <TableCell>{t.logs.protocol}</TableCell>
                    <TableCell>{t.logs.method}</TableCell>
                    <TableCell>{t.logs.status}</TableCell>
                    <TableCell>{t.logs.latency}</TableCell>
                    <TableCell>{t.logs.tokens}</TableCell>
                    <TableCell>{t.logs.cost}</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {logs.map((log) => (
                    <TableRow
                      key={log.id}
                      hover
                      onClick={() => handleOpenDetail(log)}
                      sx={{ cursor: 'pointer' }}
                    >
                      <TableCell>{formatTime(log.timestamp)}</TableCell>
                      <TableCell>{log.upstream_model || log.model}</TableCell>
                      <TableCell>{log.protocol}</TableCell>
                      <TableCell>{log.stream ? t.logs.streaming : t.logs.normal}</TableCell>
                      <TableCell>
                        {log.error_type || log.status_code >= 400 ? (
                          <Chip size="small" color="error" label={t.logs.error} />
                        ) : (
                          <Chip size="small" color="success" label={t.logs.success} />
                        )}
                      </TableCell>
                      <TableCell>{log.latency_ms}ms</TableCell>
                      <TableCell>{log.prompt_tokens} &rarr; {log.completion_tokens}</TableCell>
                      <TableCell>${log.cost.toFixed(4)}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>

              <Box sx={{ display: 'flex', justifyContent: 'flex-end', alignItems: 'center', gap: 1, mt: 2 }}>
                <IconButton size="small" disabled={offset === 0} onClick={() => setOffset(Math.max(0, offset - limit))}>
                  <ChevronLeftIcon />
                </IconButton>
                <Typography variant="body2">{offset + 1} - {offset + logs.length}</Typography>
                <IconButton size="small" disabled={logs.length < limit} onClick={() => setOffset(offset + limit)}>
                  <ChevronRightIcon />
                </IconButton>
              </Box>
            </>
          )}
        </CardContent>
      </Card>

      <Dialog open={detailOpen} onClose={() => setDetailOpen(false)} maxWidth="md" fullWidth>
        <DialogTitle>{t.logs.detail}</DialogTitle>
        <DialogContent dividers>
          {selectedLog && (
            <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
              {formatTime(selectedLog.timestamp)} &middot; {selectedLog.upstream_model || selectedLog.model} &middot; {selectedLog.protocol} &middot; {selectedLog.latency_ms}ms
            </Typography>
          )}
          <Tabs value={detailTab} onChange={(_, v) => setDetailTab(v)}>
            <Tab label={t.logs.request} />
            <Tab label={t.logs.response} />
          </Tabs>
          <Box sx={{ mt: 2 }}>
            <Box
              component="pre"
              sx={{
                bgcolor: 'grey.100',
                p: 2,
                borderRadius: 1,
                fontFamily: 'monospace',
                fontSize: '12px',
                maxHeight: 400,
                overflow: 'auto',
                whiteSpace: 'pre-wrap',
                wordBreak: 'break-word',
              }}
            >
              {detailTab === 0
                ? detailBody.request || '...'
                : detailBody.response || '...'}
            </Box>
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDetailOpen(false)}>{t.common.close}</Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}
